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

use kitchensink_runtime::AccountId;
use sp_keyring::AccountKeyring;
use std::{thread, time::Duration};
use substrate_api_client::{
	ac_node_api::EventDetails,
	ac_primitives::{
		AssetRuntimeConfig, Config, ExtrinsicSigner as GenericExtrinsicSigner, SignExtrinsic,
	},
	extrinsic::BalancesExtrinsics,
	rpc::{HandleSubscription, JsonrpseeClient},
	Api, SubmitAndWatch, SubmitExtrinsic, TransactionStatus, XtStatus,
};

type ExtrinsicSigner = GenericExtrinsicSigner<AssetRuntimeConfig>;
type ExtrinsicAddressOf<Signer> = <Signer as SignExtrinsic<AccountId>>::ExtrinsicAddress;
type Hash = <AssetRuntimeConfig as Config>::Hash;

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().unwrap();
	let alice_pair = AccountKeyring::Alice.pair();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).unwrap();

	api.set_signer(ExtrinsicSigner::new(alice_pair));

	let bob: ExtrinsicAddressOf<ExtrinsicSigner> = AccountKeyring::Bob.to_account_id().into();

	// Submit extrinsic.
	let xt0 = api.balance_transfer_allow_death(bob.clone(), 1000);
	let _tx_hash = api.submit_extrinsic(xt0).unwrap();

	// Submit and watch.
	thread::sleep(Duration::from_secs(6)); // Wait a little to avoid transaction too low priority error.
	let api1 = api.clone();
	let xt1 = api.balance_transfer_allow_death(bob.clone(), 1000);
	let watch_handle = thread::spawn(move || {
		let mut tx_subscription = api1.submit_and_watch_extrinsic(xt1).unwrap();
		let tx_status = tx_subscription.next().unwrap().unwrap();
		assert!(matches!(tx_status, TransactionStatus::Ready));
		let tx_status = tx_subscription.next().unwrap().unwrap();
		assert!(matches!(tx_status, TransactionStatus::InBlock(_)));
		let tx_status = tx_subscription.next().unwrap().unwrap();
		assert!(matches!(tx_status, TransactionStatus::Finalized(_)));
		tx_subscription.unsubscribe().unwrap();
		println!("Success: submit_and_watch_extrinsic");
	});

	// Test different _watch_untils with events
	thread::sleep(Duration::from_secs(6)); // Wait a little to avoid transaction too low priority error.
	let xt2 = api.balance_transfer_allow_death(bob.clone(), 1000);
	let report = api.submit_and_watch_extrinsic_until(xt2, XtStatus::Ready).unwrap();
	assert!(report.block_hash.is_none());
	assert!(matches!(report.status, TransactionStatus::Ready));
	assert!(report.events.is_none());
	println!("Success: submit_and_watch_extrinsic_until Ready");

	thread::sleep(Duration::from_secs(6)); // Wait a little to avoid transaction too low priority error.
	let xt3 = api.balance_transfer_allow_death(bob.clone(), 1000);
	let report = api.submit_and_watch_extrinsic_until(xt3, XtStatus::Broadcast).unwrap();
	// The xt is not broadcast - we only have one node running. Therefore, InBlock is returned.
	assert!(report.block_hash.is_some());
	assert!(matches!(report.status, TransactionStatus::InBlock(_)));
	// But we still don't fetch events, since we originally only waited for Broadcast.
	assert!(report.events.is_none());
	println!("Success: submit_and_watch_extrinsic_until Broadcast");

	let api2 = api.clone();
	thread::sleep(Duration::from_secs(6)); // Wait a little to avoid transaction too low priority error.
	let xt4 = api2.balance_transfer_allow_death(bob.clone(), 1000);
	let until_in_block_handle = thread::spawn(move || {
		let report = api2.submit_and_watch_extrinsic_until(xt4, XtStatus::InBlock).unwrap();
		assert!(report.block_hash.is_some());
		assert!(matches!(report.status, TransactionStatus::InBlock(_)));
		assert_associated_events_match_expected(report.events.unwrap());
		println!("Success: submit_and_watch_extrinsic_until InBlock");
	});

	let api3 = api.clone();
	thread::sleep(Duration::from_secs(6)); // Wait a little to avoid transaction too low priority error.
	let xt5 = api.balance_transfer_allow_death(bob.clone(), 1000);
	let until_finalized_handle = thread::spawn(move || {
		let report = api3.submit_and_watch_extrinsic_until(xt5, XtStatus::Finalized).unwrap();
		assert!(report.block_hash.is_some());
		assert!(matches!(report.status, TransactionStatus::Finalized(_)));
		assert_associated_events_match_expected(report.events.unwrap());
		println!("Success: submit_and_watch_extrinsic_until Finalized");
	});

	// Test some _watch_untils_without_events. One is enough, because it is tested implicitly by `submit_and_watch_extrinsic_until`
	// as internal call.
	thread::sleep(Duration::from_secs(6)); // Wait a little to avoid transaction too low priority error.
	let xt6 = api.balance_transfer_allow_death(bob.clone(), 1000);
	let report = api
		.submit_and_watch_extrinsic_until_without_events(xt6, XtStatus::Ready)
		.unwrap();
	assert!(report.block_hash.is_none());
	assert!(report.events.is_none());
	println!("Success: submit_and_watch_extrinsic_until_without_events Ready!");

	thread::sleep(Duration::from_secs(6)); // Wait a little to avoid transaction too low priority error.
	let xt7 = api.balance_transfer_allow_death(bob, 1000);
	let report = api
		.submit_and_watch_extrinsic_until_without_events(xt7, XtStatus::InBlock)
		.unwrap();
	println!("Extrinsic got successfully included in Block!");
	assert!(report.block_hash.is_some());
	assert!(report.events.is_none());

	watch_handle.join().unwrap();
	until_in_block_handle.join().unwrap();
	until_finalized_handle.join().unwrap();
}

fn assert_associated_events_match_expected(events: Vec<EventDetails<Hash>>) {
	// First event
	assert_eq!(events[0].pallet_name(), "Balances");
	assert_eq!(events[0].variant_name(), "Withdraw");

	assert_eq!(events[1].pallet_name(), "Balances");
	assert_eq!(events[1].variant_name(), "Transfer");

	assert_eq!(events[2].pallet_name(), "Balances");
	assert_eq!(events[2].variant_name(), "Deposit");

	assert_eq!(events[3].pallet_name(), "Treasury");
	assert_eq!(events[3].variant_name(), "Deposit");

	assert_eq!(events[4].pallet_name(), "Balances");
	assert_eq!(events[4].variant_name(), "Deposit");

	assert_eq!(events[5].pallet_name(), "TransactionPayment");
	assert_eq!(events[5].variant_name(), "TransactionFeePaid");

	assert_eq!(events[6].pallet_name(), "System");
	assert_eq!(events[6].variant_name(), "ExtrinsicSuccess");
}
