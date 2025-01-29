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

//! Tests for the chain rpc interface functions, including testing the RococoRuntimeConfig
//! and Signer generation for the RococoRuntimeConfig.

use sp_keyring::Sr25519Keyring;
use substrate_api_client::{
	ac_primitives::RococoRuntimeConfig,
	rpc::{HandleSubscription, JsonrpseeClient},
	Api, GetChainInfo, SubscribeChain,
};

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<RococoRuntimeConfig, _>::new(client).await.unwrap();
	let signer = Sr25519Keyring::Alice.pair();
	api.set_signer(signer.into());

	// GetChainInfo
	let finalized_header_hash = api.get_finalized_head().await.unwrap().unwrap();
	let _latest_header = api.get_header(None).await.unwrap().unwrap();
	let _some_header = api.get_header(Some(finalized_header_hash)).await.unwrap().unwrap();
	let _block_hash = api.get_block_hash(None).await.unwrap().unwrap();
	let block_hash = api.get_block_hash(Some(1)).await.unwrap().unwrap();
	let _block = api.get_block(None).await.unwrap().unwrap();
	let _block = api.get_block(Some(block_hash)).await.unwrap().unwrap();
	let _block = api.get_block_by_num(None).await.unwrap().unwrap();
	let _block = api.get_block_by_num(Some(2)).await.unwrap().unwrap();
	let _signed_block = api.get_signed_block(None).await.unwrap().unwrap();
	let _signed_block = api.get_signed_block(Some(block_hash)).await.unwrap().unwrap();
	let _signed_block = api.get_signed_block_by_num(None).await.unwrap().unwrap();
	let _signed_block = api.get_signed_block_by_num(Some(1)).await.unwrap().unwrap();

	// Subscription
	let mut finalized_head_subscription = api.subscribe_finalized_heads().await.unwrap();
	let _some_head = finalized_head_subscription.next().await.unwrap().unwrap();
}
