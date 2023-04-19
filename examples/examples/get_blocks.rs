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

//! Very simple example that shows how to pretty print the metadata. Has proven to be a helpful
//! debugging tool.

use kitchensink_runtime::Runtime;
use sp_core::sr25519;
use substrate_api_client::{
	ac_primitives::PlainTipExtrinsicParams,
	rpc::{HandleSubscription, JsonrpseeClient},
	Api, GetChainInfo, SubscribeChain,
};

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api.
	let client = JsonrpseeClient::with_default_url().unwrap();
	let api =
		Api::<sr25519::Pair, _, PlainTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();

	println!("Genesis block: \n {:?} \n", api.get_genesis_block().unwrap());

	let header_hash = api.get_finalized_head().unwrap().unwrap();
	println!("Latest Finalized Header Hash:\n {} \n", header_hash);

	let header = api.get_header(Some(header_hash)).unwrap().unwrap();
	println!("Finalized header:\n {:?} \n", header);

	let signed_block = api.get_finalized_block().unwrap().unwrap();
	println!("Finalized block:\n {:?} \n", signed_block);

	let last_block_number = signed_block.block.header.number;
	// Get the previous three blocks of the last_block_number
	let number_of_last_three_blocks: Vec<_> =
		(last_block_number.saturating_sub(3)..last_block_number).collect();
	let blocks = api.get_signed_blocks(&number_of_last_three_blocks).unwrap();
	println!("Block numbers of the previous three blocks: ");
	for (i, b) in blocks.iter().enumerate() {
		println!("  Block {} has block number {}", i, b.block.header.number);
	}

	println!("Latest Header: \n {:?} \n", api.get_header(None).unwrap());

	println!("Latest block: \n {:?} \n", api.get_block(None).unwrap());

	println!("Subscribing to finalized heads");
	let mut subscription = api.subscribe_finalized_heads().unwrap();

	for _ in 0..5 {
		let head = subscription.next().unwrap().unwrap();
		println!("Got new Block {:?}", head);
	}
}
