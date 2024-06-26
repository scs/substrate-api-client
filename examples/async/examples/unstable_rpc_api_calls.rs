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

//! This example shows how to call the unstable rpc api with self defined functions.
//! This includes simple requests as well as subscription.

use codec::Encode;
use serde_json::Value;
use sp_core::Bytes;
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	ac_compose_macros::rpc_params,
	ac_primitives::AssetRuntimeConfig,
	extrinsic::BalancesExtrinsics,
	rpc::{HandleSubscription, JsonrpseeClient, Request, Subscribe},
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

	// Retrieve all available rpc methods:
	let json_value: Value = api.client().request("rpc_methods", rpc_params![]).await.unwrap();
	let json_string = serde_json::to_string(&json_value).unwrap();
	println!("Available methods: {json_string} \n");

	// Since it's an unstable api and might change anytime, we first check if our calls are still
	// available:
	let chain_name_request = "chainSpec_v1_chainName";
	let chain_genesis_hash_request = "chainSpec_v1_genesisHash";
	let transaction_submit_watch = "transaction_unstable_submitAndWatch";
	let transaction_unwatch = "transaction_unstable_unwatch";

	let request_vec = [
		chain_name_request,
		chain_genesis_hash_request,
		transaction_submit_watch,
		transaction_unwatch,
	];
	for request in request_vec {
		if !json_string.contains(request) {
			panic!("Api has changed, please update the call {request}.");
		}
	}

	// Submit the above defiend rpc requests:
	let chain_name: String = api.client().request(chain_name_request, rpc_params![]).await.unwrap();
	println!("Our chain is called: {chain_name}");

	let genesishash: String =
		api.client().request(chain_genesis_hash_request, rpc_params![]).await.unwrap();
	println!("Chain genesis Hash: {genesishash}");

	// Submit and watch a transaction:
	let bob = AccountKeyring::Bob.to_account_id();
	let encoded_extrinsic: Bytes = api
		.balance_transfer_allow_death(bob.into(), 1000)
		.await
		.unwrap()
		.encode()
		.into();

	let mut subscription = api
		.client()
		.subscribe::<Value>(
			transaction_submit_watch,
			rpc_params![encoded_extrinsic],
			transaction_unwatch,
		)
		.await
		.unwrap();
	while let Some(notification) = subscription.next().await {
		let notification = notification.unwrap();
		println!("Subscription notification: {notification:?}");
		let event_object_string = notification["event"].as_str().unwrap();
		//let event_object_string = serde_json::from_string().unwrap();
		match event_object_string {
			"finalized" => break,
			"bestChainBlockIncluded" | "validated" => println!("Got {event_object_string} event"),
			_ => panic!("Unexpected event: {event_object_string}"),
		};
	}
	println!("Transaction got finalized, unsubscribing.");
	subscription.unsubscribe().await.unwrap();
}
