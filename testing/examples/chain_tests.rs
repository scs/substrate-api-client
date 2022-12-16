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

//! Tests for the chain rpc interface functions.

use kitchensink_runtime::Runtime;
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	rpc::{HandleSubscription, JsonrpseeClient},
	Api, AssetTipExtrinsicParams, GetBlock, GetHeader, SubscribeChain,
};

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().unwrap();
	let alice_pair = AccountKeyring::Alice.pair();
	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(alice_pair);

	// GetHeader
	let finalized_header_hash = api.get_finalized_head().unwrap().unwrap();
	let _latest_header = api.get_header(None).unwrap().unwrap();
	let _some_header = api.get_header(Some(finalized_header_hash)).unwrap().unwrap();

	// GetBlock
	let _block_hash = api.get_block_hash(None).unwrap().unwrap();
	let block_hash = api.get_block_hash(Some(1)).unwrap().unwrap();
	let _block = api.get_block(None).unwrap().unwrap();
	let _block = api.get_block(Some(block_hash)).unwrap().unwrap();
	let _block = api.get_block_by_num(None).unwrap().unwrap();
	let _block = api.get_block_by_num(Some(2)).unwrap().unwrap();
	let _signed_block = api.get_signed_block(None).unwrap().unwrap();
	let _signed_block = api.get_signed_block(Some(block_hash)).unwrap().unwrap();
	let _signed_block = api.get_signed_block_by_num(None).unwrap().unwrap();
	let _signed_block = api.get_signed_block_by_num(Some(1)).unwrap().unwrap();

	// Subscription
	let mut finalized_head_subscription = api.subscribe_finalized_heads().unwrap();
	let _some_head = finalized_head_subscription.next().unwrap().unwrap();
}
