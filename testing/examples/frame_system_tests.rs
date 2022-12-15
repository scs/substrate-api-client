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
use kitchensink_runtime::Runtime;
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	rpc::JsonrpseeClient, Api, AssetTipExtrinsicParams, GetAccountInformation, StaticEvent,
	SubscribeEvents, SubscribeFrameSystem,
};

/// Check out frame_system::Event::ExtrinsicSuccess:
#[derive(Decode)]
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
	api.set_signer(alice_pair);

	let alice = AccountKeyring::Alice.to_account_id();

	// GetAccountInformation
	let _account_info = api.get_account_info(&alice).unwrap().unwrap();
	let _account_data = api.get_account_data(&alice).unwrap().unwrap();

	// Subscribe
	let mut event_subscription = api.subscribe_system_events().unwrap();
	let _event: ExtrinsicSuccess = api.wait_for_event(&mut event_subscription).unwrap();
	let _event_details =
		api.wait_for_event_details::<ExtrinsicSuccess>(&mut event_subscription).unwrap();
	println!("Success: Wait for event Details");
}
