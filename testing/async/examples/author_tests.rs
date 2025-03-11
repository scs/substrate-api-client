/*
	Copyright 2019 Supercomputing Systems AG
	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.
*/

//! Tests for the author rpc interface functions.

use resonance_runtime::{BalancesCall, RuntimeCall};
use sp_core::{Encode, H256};
use sp_keyring::Sr25519Keyring;
use substrate_api_client::{
	ac_node_api::RawEventDetails,
	ac_primitives::{
		Config, ExtrinsicSigner as GenericExtrinsicSigner, RococoRuntimeConfig, SignExtrinsic,
	},
	rpc::{HandleSubscription, JsonrpseeClient},
	Api, SubmitAndWatch, SubmitExtrinsic, TransactionStatus, XtStatus,
};

type ExtrinsicSigner = GenericExtrinsicSigner<RococoRuntimeConfig>;
type ExtrinsicAddressOf<Signer> = <Signer as SignExtrinsic<AccountId>>::ExtrinsicAddress;
type Hash = <RococoRuntimeConfig as Config>::Hash;
type MyApi = Api<RococoRuntimeConfig, JsonrpseeClient>;
type Index = <RococoRuntimeConfig as Config>::Index;
type AccountId = <RococoRuntimeConfig as Config>::AccountId;

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let alice_pair = Sr25519Keyring::Alice.pair();
	let mut api = MyApi::new(client).await.unwrap();
	api.set_signer(alice_pair.into());
	let bob: ExtrinsicAddressOf<ExtrinsicSigner> = Sr25519Keyring::Bob.to_account_id().into();
	let signer_nonce = api.get_nonce().await.unwrap();
	let transfer_call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
		dest: bob.clone(),
		value: 1000,
	});

	// Submit_extrinsic.
	let xt0 = api.compose_extrinsic_offline(transfer_call.clone(), signer_nonce);
	let _tx_hash = api.submit_extrinsic(xt0).await.unwrap();

	tokio::join!(
		test_submit_and_watch(&api, transfer_call.clone(), signer_nonce + 1),
		test_submit_and_watch_until_ready(&api, transfer_call.clone(), signer_nonce + 2),
		test_submit_and_watch_until_broadcast(&api, transfer_call.clone(), signer_nonce + 3),
		test_submit_and_watch_until_in_block(&api, transfer_call.clone(), signer_nonce + 4),
		test_submit_and_watch_until_finalized(&api, transfer_call.clone(), signer_nonce + 5),
		test_submit_and_watch_until_retracted(&api, transfer_call.clone(), signer_nonce + 6),
		// Test some _watch_untils_without_events. We don't need to test all, because it is tested implicitly by `submit_and_watch_extrinsic_until`
		// as internal call.
		test_submit_and_watch_extrinsic_until_ready_without_events(
			&api,
			transfer_call.clone(),
			signer_nonce + 7
		),
		test_submit_and_watch_extrinsic_until_in_block_without_events(
			&api,
			transfer_call.clone(),
			signer_nonce + 8
		)
	);
}

async fn test_submit_and_watch(api: &MyApi, transfer_call: RuntimeCall, nonce: Index) {
	let xt = api.compose_extrinsic_offline(transfer_call, nonce);

	let mut tx_subscription = api.submit_and_watch_extrinsic(xt).await.unwrap();
	let tx_status = tx_subscription.next().await.unwrap().unwrap();
	assert!(matches!(tx_status, TransactionStatus::Ready));
	let tx_status = tx_subscription.next().await.unwrap().unwrap();
	assert!(matches!(tx_status, TransactionStatus::InBlock(_)));
	let tx_status = tx_subscription.next().await.unwrap().unwrap();
	assert!(matches!(tx_status, TransactionStatus::Finalized(_)));
	tx_subscription.unsubscribe().await.unwrap();
	println!("Success: submit_and_watch_extrinsic: {:?}", tx_status);
}

async fn test_submit_and_watch_until_ready(api: &MyApi, transfer_call: RuntimeCall, nonce: Index) {
	std::thread::sleep(std::time::Duration::from_secs(1));
	let xt = api.compose_extrinsic_offline(transfer_call, nonce);
	let extrinsic_hash: H256 = sp_crypto_hashing::blake2_256(&xt.encode()).into();
	let report = api.submit_and_watch_extrinsic_until(xt, XtStatus::Ready).await.unwrap();
	assert_eq!(extrinsic_hash, report.extrinsic_hash);
	assert!(report.block_hash.is_none());
	assert!(matches!(report.status, TransactionStatus::Ready));
	assert!(report.events.is_none());
	println!("Success: submit_and_watch_extrinsic_until {:?}", report.status);
}

async fn test_submit_and_watch_until_broadcast(
	api: &MyApi,
	transfer_call: RuntimeCall,
	nonce: Index,
) {
	std::thread::sleep(std::time::Duration::from_secs(1));
	let xt = api.compose_extrinsic_offline(transfer_call, nonce);
	let report = api.submit_and_watch_extrinsic_until(xt, XtStatus::Broadcast).await.unwrap();
	// The xt is not broadcast - we only have one node running. Therefore, InBlock is returned.
	assert!(report.block_hash.is_some());
	assert!(matches!(report.status, TransactionStatus::InBlock(_)));
	// But we still don't fetch events, since we originally only waited for Broadcast.
	assert!(report.events.is_none());
	println!("Success: submit_and_watch_extrinsic_until {:?}", report.status);
}

async fn test_submit_and_watch_until_in_block(
	api: &MyApi,
	transfer_call: RuntimeCall,
	nonce: Index,
) {
	std::thread::sleep(std::time::Duration::from_secs(1));
	let xt = api.compose_extrinsic_offline(transfer_call, nonce);
	let report = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).await.unwrap();
	assert!(report.block_hash.is_some());
	assert!(matches!(report.status, TransactionStatus::InBlock(_)));
	assert_associated_events_match_expected(&report.events.unwrap());
	println!("Success: submit_and_watch_extrinsic_until {:?}", report.status);
}

async fn test_submit_and_watch_until_finalized(
	api: &MyApi,
	transfer_call: RuntimeCall,
	nonce: Index,
) {
	std::thread::sleep(std::time::Duration::from_secs(1));
	let xt = api.compose_extrinsic_offline(transfer_call, nonce);
	let report = api.submit_and_watch_extrinsic_until(xt, XtStatus::Finalized).await.unwrap();
	assert!(report.block_hash.is_some());
	assert!(matches!(report.status, TransactionStatus::Finalized(_)));
	assert_associated_events_match_expected(&report.events.unwrap());
	println!("Success: submit_and_watch_extrinsic_until {:?}", report.status);
}

async fn test_submit_and_watch_until_retracted(
	api: &MyApi,
	transfer_call: RuntimeCall,
	nonce: Index,
) {
	std::thread::sleep(std::time::Duration::from_secs(1));
	let xt = api.compose_extrinsic_offline(transfer_call, nonce);
	// We wait for `Retracted`` but we cannot simulate this in a test. Therefore we will receive the status after `Retracted`
	// which is `Finalized`
	let report = api.submit_and_watch_extrinsic_until(xt, XtStatus::Retracted).await.unwrap();
	assert!(report.block_hash.is_some());
	assert!(matches!(report.status, TransactionStatus::Finalized(_)));
	assert_associated_events_match_expected(&report.events.unwrap());
	println!("Success: submit_and_watch_extrinsic_until {:?}", report.status);
}

async fn test_submit_and_watch_extrinsic_until_ready_without_events(
	api: &MyApi,
	transfer_call: RuntimeCall,
	nonce: Index,
) {
	std::thread::sleep(std::time::Duration::from_secs(1));
	let xt = api.compose_extrinsic_offline(transfer_call, nonce);
	let report = api
		.submit_and_watch_extrinsic_until_without_events(xt, XtStatus::Ready)
		.await
		.unwrap();
	assert!(report.block_hash.is_none());
	assert!(report.events.is_none());
	println!("Success: submit_and_watch_extrinsic_until_without_events {:?}", report.status);
}

async fn test_submit_and_watch_extrinsic_until_in_block_without_events(
	api: &MyApi,
	transfer_call: RuntimeCall,
	nonce: Index,
) {
	// Wait a little, otherwise we may run into future
	std::thread::sleep(std::time::Duration::from_secs(1));
	let xt = api.compose_extrinsic_offline(transfer_call, nonce);
	let mut report = api
		.submit_and_watch_extrinsic_until_without_events(xt, XtStatus::InBlock)
		.await
		.unwrap();
	println!("Extrinsic got successfully included in Block!");
	assert!(report.block_hash.is_some());
	assert!(report.events.is_none());

	// Should fail without events
	assert!(report.check_events_for_dispatch_error(&api.metadata()).is_err());

	// Now we fetch the events separately
	api.populate_events(&mut report).await.unwrap();
	assert!(report.events.is_some());
	assert!(report.check_events_for_dispatch_error(&api.metadata()).is_ok());
	let events = report.events.as_ref().unwrap();
	assert_associated_events_match_expected(&events);

	// Can populate events only once
	let result = api.populate_events(&mut report).await;
	assert!(result.is_err());
}

fn assert_associated_events_match_expected(events: &[RawEventDetails<Hash>]) {
	// First event
	assert_eq!(events[0].pallet_name(), "Balances");
	assert_eq!(events[0].variant_name(), "Withdraw");

	assert_eq!(events[1].pallet_name(), "Balances");
	assert_eq!(events[1].variant_name(), "Transfer");

	assert_eq!(events[2].pallet_name(), "Balances");
	assert_eq!(events[2].variant_name(), "Deposit");

	assert_eq!(events[3].pallet_name(), "Balances");
	assert_eq!(events[3].variant_name(), "Deposit");

	assert_eq!(events[4].pallet_name(), "TransactionPayment");
	assert_eq!(events[4].variant_name(), "TransactionFeePaid");

	assert_eq!(events[5].pallet_name(), "System");
	assert_eq!(events[5].variant_name(), "ExtrinsicSuccess");
}
