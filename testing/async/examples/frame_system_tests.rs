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
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	ac_node_api::StaticEvent, ac_primitives::AssetRuntimeConfig, rpc::JsonrpseeClient, Api,
	GetAccountInformation, SystemApi,
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
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let alice_pair = AccountKeyring::Alice.pair();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).await.unwrap();
	api.set_signer(alice_pair.into());

	let alice = AccountKeyring::Alice.to_account_id();

	// GetAccountInformation
	let _account_info = api.get_account_info(&alice).await.unwrap().unwrap();
	let _account_data = api.get_account_data(&alice).await.unwrap().unwrap();

	// Empty account information
	let inexistent_account = AccountKeyring::Two.to_account_id();
	let maybe_account_info = api.get_account_info(&inexistent_account).await.unwrap();
	assert!(maybe_account_info.is_none());
	let maybe_account_data = api.get_account_data(&inexistent_account).await.unwrap();
	assert!(maybe_account_data.is_none());

	// System Api
	let next_index = api.get_system_account_next_index(alice).await.unwrap();
	// Alice has not yet sent any extrinsic, so next_index should be 0.
	assert_eq!(next_index, 0);

	let system_name = api.get_system_name().await.unwrap();
	println!("System name: {system_name}");

	let system_version = api.get_system_version().await.unwrap();
	println!("System version: {system_version}");

	let system_chain = api.get_system_chain().await.unwrap();
	println!("System chain: {system_chain}");

	let system_chain_type = api.get_system_chain_type().await.unwrap();
	println!("System chain type: {system_chain_type:?}");

	let system_properties = api.get_system_properties().await.unwrap();
	println!("System properties: {system_properties:?}");

	let system_health = api.get_system_health().await.unwrap();
	println!("System health: {system_health}");

	let system_local_peer_id = api.get_system_local_peer_id().await.unwrap();
	println!("System local peer id: {system_local_peer_id:?}");

	let system_local_listen_addresses = api.get_system_local_listen_addresses().await.unwrap();
	println!("System local listen addresses: {system_local_listen_addresses:?}");
}
