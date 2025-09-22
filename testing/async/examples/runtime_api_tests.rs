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

//! Tests for the runtime api.

use sp_core::{sr25519, Decode};
use sp_keyring::Sr25519Keyring;
use substrate_api_client::{
	ac_primitives::RococoRuntimeConfig,
	extrinsic::BalancesExtrinsics,
	rpc::JsonrpseeClient,
	runtime_api::{
		AccountNonceApi, AuthorityDiscoveryApi, BlockBuilderApi, CoreApi, MetadataApi, RuntimeApi,
		TransactionPaymentApi,
	},
	Api, GetChainInfo,
};

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let alice_pair = Sr25519Keyring::Alice.pair();
	let mut api = Api::<RococoRuntimeConfig, _>::new(client).await.unwrap();
	api.set_signer(alice_pair.into());

	let runtime_api = api.runtime_api();

	let alice = Sr25519Keyring::Alice.to_account_id();
	let bob = Sr25519Keyring::Bob.to_account_id();

	// General Runtime Api
	let bytes = runtime_api.rpc_call("Metadata_metadata_versions", None, None).await.unwrap();
	let metadata_versions = Vec::<u32>::decode(&mut bytes.0.as_slice()).unwrap();
	assert_eq!(metadata_versions, [14, 15, 16]);

	// AccountNonce
	let alice_nonce = runtime_api.account_nonce(alice, None).await.unwrap();
	assert_eq!(alice_nonce, api.get_nonce().await.unwrap());

	// Authority Discovery
	let authority_id: Vec<sr25519::Public> = runtime_api.authorities(None).await.unwrap();
	assert!(authority_id.len() > 0);

	// BlockBuilder
	let extrinsic = api.balance_transfer_allow_death(bob.clone().into(), 1000).await.unwrap();
	runtime_api.apply_extrinsic(extrinsic, None).await.unwrap().unwrap().unwrap();
	let block = api.get_block_by_num(Some(0)).await.unwrap().unwrap();
	let check = runtime_api.check_inherents(block, Default::default(), None).await.unwrap();
	assert!(check.ok());
	// This doesn't seem to work with the current substrate node. Tried it on polkadot.js as well, but it keeps on runtime panicking.
	//let _bytes = runtime_api.inherent_extrinsics(Default::default(), None).unwrap();
	//let _header = runtime_api.finalize_block(None).unwrap();

	// Core
	let _version = runtime_api.version(None).await.unwrap();

	// Metadata
	let _metadata = runtime_api.metadata(None).await.unwrap();
	let _metadata = runtime_api.metadata_at_version(15, None).await.unwrap().unwrap();
	let _method_names = runtime_api.list_methods_of_trait("BabeApi", None).await.unwrap();
	let _trait_names = runtime_api.list_traits(None).await.unwrap();
	let metadata_versions = runtime_api.metadata_versions(None).await.unwrap();
	assert_eq!(metadata_versions, [14, 15, 16]);

	// MMR
	// This doesn't seem to work with the current substrate node. Tried it on polkadot.js aswell, but it keeps on runtime panicking.
	// let generated_proof = runtime_api.generate_proof(vec![0, 1], None, None).unwrap().unwrap();
	// let root = runtime_api.root(None).unwrap().unwrap();
	// runtime_api
	// 	.verify_proof(generated_proof.0, generated_proof.1, None)
	// 	.unwrap()
	// 	.unwrap();
	// let generated_proof = runtime_api.generate_proof(vec![1], None, None).unwrap().unwrap();
	// runtime_api
	// 	.verify_proof_stateless(root[0], generated_proof.0, generated_proof.1, None)
	// 	.unwrap()
	// 	.unwrap();

	// Sessions keys
	// This doesn't seem to work with the current substrate node. Tried it on polkadot.js aswell, but it keeps on runtime panicking.
	// let encoded_session_keys = runtime_api.generate_session_keys(None, None).unwrap();
	// let _session_keys =
	// 	runtime_api.decode_session_keys(encoded_session_keys, None).unwrap().unwrap();

	// Staking not available
	// let _quota = runtime_api.nominations_quota(100000000, None).await.unwrap();

	// Transaction Payment
	let extrinsic = api.balance_transfer_allow_death(bob.clone().into(), 1000).await.unwrap();
	let _tx_fee_details =
		runtime_api.query_fee_details(extrinsic.clone(), 1000, None).await.unwrap();
	let _tx_info = runtime_api.query_info(extrinsic, 1000, None).await.unwrap();
	let _fee = runtime_api.query_length_to_fee(1000, None).await.unwrap();
	let _fee = runtime_api.query_weight_to_fee(1000.into(), None).await.unwrap();

	// Transaction Payment Call not available on rococo runtime.
	// let call = api
	// 	.balance_transfer_allow_death(bob.clone().into(), 1000)
	// 	.await
	// 	.unwrap()
	// 	.function;
	// let _tx_fee_details =
	// 	runtime_api.query_call_fee_details(call.clone(), 1000, None).await.unwrap();
	// let _tx_info = runtime_api.query_call_info(call, 1000, None).await.unwrap();
	// let _fee = runtime_api.query_length_to_fee_call(1000, None).await.unwrap();
	// let _fee = runtime_api.query_weight_to_fee_call(1000.into(), None).await.unwrap();
}
