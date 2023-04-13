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

use sp_runtime::traits::GetRuntimeBlockType;
use substrate_api_client::{
	ac_primitives::SubstrateConfig,
	rpc::{HandleSubscription, JsonrpseeClient},
	Api, GetBlock, GetHeader, SubscribeChain,
};

// This example run against a specific  node.
// We use the substrate kitchensink runtime: the config is a substrate config with the kitchensink runtime block type.
// ! Careful: Most runtimes uses plain as tips, they need a polkadot config.
// For better code readability, we define the config type.
type KitchensinkConfig =
	SubstrateConfig<<kitchensink_runtime::Runtime as GetRuntimeBlockType>::RuntimeBlock>;

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api.
	let client = JsonrpseeClient::with_default_url().unwrap();
	let api = Api::<KitchensinkConfig, _>::new(client).unwrap();

	println!("Genesis block: \n {:?} \n", api.get_block_by_num(Some(0)).unwrap());

	let header_hash = api.get_finalized_head().unwrap().unwrap();
	println!("Latest Finalized Header Hash:\n {} \n", header_hash);

	let h = api.get_header(Some(header_hash)).unwrap().unwrap();
	println!("Finalized header:\n {:?} \n", h);

	let b = api.get_signed_block(Some(header_hash)).unwrap().unwrap();
	println!("Finalized signed block:\n {:?} \n", b);

	println!("Latest Header: \n {:?} \n", api.get_header(None).unwrap());

	println!("Latest block: \n {:?} \n", api.get_block(None).unwrap());

	println!("Subscribing to finalized heads");
	let mut subscription = api.subscribe_finalized_heads().unwrap();

	for _ in 0..5 {
		let head = subscription.next().unwrap().unwrap();
		println!("Got new Block {:?}", head);
	}
}
