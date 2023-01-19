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

use kitchensink_runtime::{AccountId, Runtime, Signature};
use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use std::{thread, time::Duration};
use substrate_api_client::{
	extrinsic::BalancesExtrinsics,
	rpc::{HandleSubscription, JsonrpseeClient},
	Api, AssetTipExtrinsicParams, EventDetails, ExtrinsicSigner as GenericExtrinsicSigner,
	SignExtrinsic, SubmitAndWatch, SubmitAndWatchUntilSuccess, SubmitExtrinsic, TransactionStatus,
	XtStatus,
};

type ExtrinsicSigner = GenericExtrinsicSigner<Pair, Signature, Runtime>;
type ExtrinsicAddressOf<Signer> = <Signer as SignExtrinsic<AccountId>>::ExtrinsicAddress;

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().unwrap();
	let alice_pair = AccountKeyring::Alice.pair();
	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(ExtrinsicSigner::new(alice_pair));

	let bob: ExtrinsicAddressOf<ExtrinsicSigner> = AccountKeyring::Bob.to_account_id().into();

	// Submit extrinisc.
	let xt0 = api.balance_transfer(bob.clone(), 1000);
	let _tx_hash = api.submit_extrinsic(xt0).unwrap();

	// Submit and watch.
	thread::sleep(Duration::from_secs(6)); // Wait a little to avoid transaction too low priority error.
	let api1 = api.clone();
	let xt1 = api.balance_transfer(bob.clone(), 1000);
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

	// Test different _watch_untils.

	thread::sleep(Duration::from_secs(6)); // Wait a little to avoid transaction too low priority error.
	let xt2 = api.balance_transfer(bob.clone(), 1000);
	let report = api.submit_and_watch_extrinsic_until(xt2, XtStatus::Ready).unwrap();
	assert!(report.block_hash.is_none());
	println!("Success: submit_and_watch_extrinsic_until Ready");

	thread::sleep(Duration::from_secs(6)); // Wait a little to avoid transaction too low priority error.
	let xt3 = api.balance_transfer(bob.clone(), 1000);
	// The xt is not broadcast - we only have one node running. Therefore, InBlock is returned.
	let _some_hash = api
		.submit_and_watch_extrinsic_until(xt3, XtStatus::Broadcast)
		.unwrap()
		.block_hash
		.unwrap();
	println!("Success: submit_and_watch_extrinsic_until Broadcast");

	let api2 = api.clone();
	thread::sleep(Duration::from_secs(6)); // Wait a little to avoid transaction too low priority error.
	let xt4 = api2.balance_transfer(bob.clone(), 1000);
	let until_in_block_handle = thread::spawn(move || {
		let _block_hash = api2
			.submit_and_watch_extrinsic_until(xt4, XtStatus::InBlock)
			.unwrap()
			.block_hash
			.unwrap();
		println!("Success: submit_and_watch_extrinsic_until InBlock");
	});

	let api3 = api.clone();
	thread::sleep(Duration::from_secs(6)); // Wait a little to avoid transaction too low priority error.
	let xt5 = api.balance_transfer(bob.clone(), 1000);
	let until_finalized_handle = thread::spawn(move || {
		let _block_hash = api3
			.submit_and_watch_extrinsic_until(xt5, XtStatus::Finalized)
			.unwrap()
			.block_hash
			.unwrap();
		println!("Success: submit_and_watch_extrinsic_until Finalized");
	});

	// Test Success.
	thread::sleep(Duration::from_secs(6)); // Wait a little to avoid transaction too low priority error.
	let xt6 = api.balance_transfer(bob, 1000);

	let events = api
		.submit_and_watch_extrinsic_until_success(xt6, false)
		.unwrap()
		.events
		.unwrap();
	println!("Extrinsic got successfully included in Block!");
	assert_assosciated_events_match_expected(events);

	watch_handle.join().unwrap();
	until_in_block_handle.join().unwrap();
	until_finalized_handle.join().unwrap();
}

fn assert_assosciated_events_match_expected(events: Vec<EventDetails>) {
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
