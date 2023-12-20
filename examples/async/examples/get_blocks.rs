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

//! Very simple example that shows how to fetch chain information with async.
//! To compile this example for async you need to set the `--no-default-features` flag

use substrate_api_client::{
	ac_primitives::AssetRuntimeConfig,
	rpc::{HandleSubscription, JsonrpseeClient},
	Api, GetChainInfo, SubscribeChain,
};

// To test this example with CI we run it against the Substrate kitchensink node, which uses the asset pallet.
// Therefore, we need to use the `AssetRuntimeConfig` in this example.
// ! However, most Substrate runtimes do not use the asset pallet at all. So if you run an example against your own node
// you most likely should use `DefaultRuntimeConfig` instead.

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api.
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let api = Api::<AssetRuntimeConfig, _>::new(client).await.unwrap();

	let (genesis_block, header_hash, signed_block) = tokio::try_join!(
		api.get_genesis_block(),
		api.get_finalized_head(),
		api.get_finalized_block(),
	)
	.unwrap();
	let header_hash = header_hash.unwrap();
	let signed_block = signed_block.unwrap();
	println!("Genesis block: \n {:?} \n", genesis_block);
	println!("Latest Finalized Header Hash:\n {} \n", header_hash);

	let last_block_number = signed_block.block.header.number;
	// Get the previous three blocks of the last_block_number
	let number_of_last_three_blocks: Vec<_> =
		(last_block_number.saturating_sub(3)..last_block_number).collect();

	let (header, blocks, latest_header, latest_block) = tokio::try_join!(
		api.get_header(Some(header_hash)),
		api.get_signed_blocks(&number_of_last_three_blocks),
		api.get_header(None),
		api.get_block(None),
	)
	.unwrap();
	println!("Finalized header:\n {:?} \n", header.unwrap());
	println!("Finalized block:\n {:?} \n", signed_block);
	println!("Block numbers of the previous three blocks: ");
	for (i, b) in blocks.iter().enumerate() {
		println!("  Block {} has block number {}", i, b.block.header.number);
	}
	println!("Latest Header: \n {:?} \n", latest_header);
	println!("Latest block: \n {:?} \n", latest_block);

	println!("Subscribing to finalized heads");
	let mut subscription = api.subscribe_finalized_heads().await.unwrap();
	for _ in 0..5 {
		let head = subscription.next().await.unwrap().unwrap();
		println!("Got new Block {:?}", head);
		println!("This print should be printed before the one with \"Got new Block\"");
	}
}
