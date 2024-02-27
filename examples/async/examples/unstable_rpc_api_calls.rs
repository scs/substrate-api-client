/*
	Copyright 2024 Supercomputing Systems AG
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

//! This example shows how to use the compose_extrinsic_offline macro which generates an extrinsic
//! without asking the node for nonce and does not need to know the metadata

use codec::Encode;
use serde_json::Value;
use sp_core::Bytes;
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	ac_compose_macros::rpc_params,
	ac_primitives::AssetRuntimeConfig,
	extrinsic::BalancesExtrinsics,
	rpc::{JsonrpseeClient, Request},
	Api,
};

// To test this example with CI we run it against the Substrate kitchensink node, which uses the asset pallet.
// Therefore, we need to use the `AssetRuntimeConfig` in this example.
// ! However, most Substrate runtimes do not use the asset pallet at all. So if you run an example against your own node
// you most likely should use `DefaultRuntimeConfig` instead.

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let signer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).await.unwrap();
	api.set_signer(signer.into());

	let json_value: Value = api.client().request("rpc_methods", rpc_params![]).await.unwrap();
	let json_string = serde_json::to_string(&json_value).unwrap();
	println!("{json_string}");

	let chain_name: String = api
		.client()
		.request("chainSpec_unstable_chainName", rpc_params![])
		.await
		.unwrap();

	println!("Our chain is called: {chain_name}");

	let genesishash: String = api
		.client()
		.request("chainSpec_unstable_genesisHash", rpc_params![])
		.await
		.unwrap();

	println!("Genesis Hash: {genesishash}");

	let bob = AccountKeyring::Bob.to_account_id();
	let encoded_extrinsic: Bytes = api
		.balance_transfer_allow_death(bob.into(), 1000)
		.await
		.unwrap()
		.encode()
		.into();

	let subscription_string: String = api
		.client()
		.request("transaction_unstable_submitAndWatch", rpc_params![encoded_extrinsic])
		.await
		.unwrap();

	println!("Successfully submitted extrinsic. Watchable with the following subscription: {subscription_string}");
}
