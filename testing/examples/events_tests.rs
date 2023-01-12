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

//! Tests for the frame system interface functions.

use codec::Decode;
use frame_support::dispatch::DispatchInfo;
use kitchensink_runtime::{Runtime, Signature};
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	rpc::JsonrpseeClient, Api, AssetTipExtrinsicParams, EventDetails, ExtrinsicSigner, FetchEvents,
	GetBlock, StaticEvent, SubmitAndWatch, XtStatus,
};

/// Check out frame_system::Event::ExtrinsicSuccess:
#[derive(Decode, Debug)]
struct ExtrinsicSuccess {
	_dispatch_info: DispatchInfo,
}

impl StaticEvent for ExtrinsicSuccess {
	const PALLET: &'static str = "System";
	const EVENT: &'static str = "ExtrinsicSuccess";
}

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().unwrap();
	let alice_pair = AccountKeyring::Alice.pair();
	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(ExtrinsicSigner::<_, Signature, Runtime>::new(alice_pair));

	let bob = AccountKeyring::Bob.to_account_id();

	// Test `fetch_events_from_block`: There should always be at least the
	// timestamp set event.
	let block_hash = api.get_block_hash(None).unwrap().unwrap();
	let events = api.fetch_events_from_block(block_hash).unwrap();
	assert!(!events.is_empty());
	println!("{events:?}");

	// Submit a test-extrinisc to test `fetch_events_for_extrinsic`.
	let xt = api.balance_transfer(bob.into(), 1000);
	let report = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).unwrap();

	let extrinisc_events = api
		.fetch_events_for_extrinsic(report.extrinsic_hash, report.block_hash.unwrap())
		.unwrap();
	assert_assosciated_events_match_expected(extrinisc_events);
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
