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

//! Very simple example that shows how to query Runtime Api of a Substrate node.

use codec::Encode;
use sp_core::sr25519;
use sp_keyring::Sr25519Keyring;
use substrate_api_client::{
	ac_primitives::RococoRuntimeConfig,
	extrinsic::BalancesExtrinsics,
	rpc::JsonrpseeClient,
	runtime_api::{AuthorityDiscoveryApi, CoreApi, MetadataApi, RuntimeApi, TransactionPaymentApi},
	Api, GetChainInfo,
};

const UNSTABLE_METADATA_VERSION: u32 = u32::MAX;

// To test this example with CI we run it against the Polkadot Rococo node. Remember to switch the Config to match your
// own runtime if it uses different parameter configurations. Several pre-compiled runtimes are available in the ac-primitives crate.

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api, which retrieves the metadata from the node upon initialization.
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<RococoRuntimeConfig, _>::new(client).await.unwrap();
	let alice_pair = Sr25519Keyring::Alice.pair();
	api.set_signer(alice_pair.into());
	let runtime_api = api.runtime_api();

	// Query the fee of an extrinsic.
	let bob = Sr25519Keyring::Bob.to_account_id();
	let balance_extrinsic =
		api.balance_transfer_allow_death(bob.clone().into(), 1000).await.unwrap();
	let extrinsic_fee_details = runtime_api
		.query_fee_details(balance_extrinsic.clone(), 1000, None)
		.await
		.unwrap();
	let final_fee = extrinsic_fee_details.final_fee();
	println!("To exceute the balance extrinsic, the following fee is required: {:?}", final_fee);

	// Get the authority Ids.
	let authority_ids: Vec<sr25519::Public> = runtime_api.authorities(None).await.unwrap();
	println!("The following authorities are currently active:");
	for authority in authority_ids {
		println!("{:?}", authority);
	}

	// Query the runtime api version.
	let version = runtime_api.version(None).await.unwrap();
	println!("{:?}", version);

	// Query the available metadata versions.
	let metadata_versions = runtime_api.metadata_versions(None).await.unwrap();
	assert_eq!(metadata_versions, [14, 15, UNSTABLE_METADATA_VERSION]);

	// List all apis and functions thereof.
	let trait_names = runtime_api.list_traits(None).await.unwrap();
	println!();
	println!("Available traits:");
	for name in trait_names {
		println!("{name}");
	}
	println!();

	let trait_name = "BabeApi";
	let method_names = runtime_api.list_methods_of_trait(trait_name, None).await.unwrap();
	println!("Available methods of {trait_name}:");
	for name in method_names {
		println!("{name}");
	}
	println!();

	// Create your own runtime api call.
	let parameters = vec![1000.encode()];
	let latest_block_hash = api.get_block_hash(None).await.unwrap().unwrap();
	let result: Result<u128, substrate_api_client::Error> = runtime_api
		.runtime_call(
			"TransactionPaymentApi_query_length_to_fee",
			parameters,
			Some(latest_block_hash),
		)
		.await;
	let output = result.unwrap();
	println!("Received the following output: {:?}", output);
}
