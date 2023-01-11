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
	rpc::JsonrpseeClient, Api, AssetTipExtrinsicParams, ExtrinsicSigner, GetAccountInformation,
	StaticEvent, SubscribeEvents, SubscribeFrameSystem, SystemApi,
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
	api.set_signer(ExtrinsicSigner::<_, Signature, Runtime>::new(alice_pair));

	let alice = AccountKeyring::Alice.to_account_id();

	// GetAccountInformation
	let _account_info = api.get_account_info(&alice).unwrap().unwrap();
	let _account_data = api.get_account_data(&alice).unwrap().unwrap();

	// System Api
	let system_name = api.get_system_name().unwrap();
	println!("System name: {}", system_name);

	let system_version = api.get_system_version().unwrap();
	println!("System version: {}", system_version);

	let system_chain = api.get_system_chain().unwrap();
	println!("System chain: {}", system_chain);

	let system_chain_type = api.get_system_chain_type().unwrap();
	println!("System chain type: {:?}", system_chain_type);

	let system_properties = api.get_system_properties().unwrap();
	println!("System properties: {:?}", system_properties);

	let system_health = api.get_system_health().unwrap();
	println!("System health: {}", system_health);

	let system_local_peer_id = api.get_system_local_peer_id().unwrap();
	println!("System local peer id: {:?}", system_local_peer_id);

	let system_local_listen_addresses = api.get_system_local_listen_addresses().unwrap();
	println!("System local listen addresses: {:?}", system_local_listen_addresses);

	// Subscribe
	let mut event_subscription = api.subscribe_system_events().unwrap();
	let _event: ExtrinsicSuccess = api.wait_for_event(&mut event_subscription).unwrap();
	let _event_details =
		api.wait_for_event_details::<ExtrinsicSuccess>(&mut event_subscription).unwrap();
	println!("Success: Wait for event Details");
}
