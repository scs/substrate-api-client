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

use kitchensink_runtime::{Block, Header};
use sp_core::sr25519;
use sp_runtime::generic::SignedBlock as SignedBlockG;
use substrate_api_client::{
	rpc::JsonrpseeClient, Api, AssetTipExtrinsicParams, HandleSubscription,
};

type SignedBlock = SignedBlockG<Block>;

fn main() {
	env_logger::init();

	let client = JsonrpseeClient::with_default_url().unwrap();
	let api = Api::<sr25519::Pair, _, AssetTipExtrinsicParams>::new(client).unwrap();

	let head = api.get_finalized_head().unwrap().unwrap();

	println!("Genesis block: \n {:?} \n", api.get_block_by_num::<Block>(Some(0)).unwrap());

	println!("Finalized Head:\n {} \n", head);

	let h: Header = api.get_header(Some(head)).unwrap().unwrap();
	println!("Finalized header:\n {:?} \n", h);

	let b: SignedBlock = api.get_signed_block(Some(head)).unwrap().unwrap();
	println!("Finalized signed block:\n {:?} \n", b);

	println!("Latest Header: \n {:?} \n", api.get_header::<Header>(None).unwrap());

	println!("Latest block: \n {:?} \n", api.get_block::<Block>(None).unwrap());

	println!("Subscribing to finalized heads");
	let mut subscription = api.subscribe_finalized_heads::<Header>().unwrap();

	for _ in 0..5 {
		let head: Header = subscription.next().unwrap().unwrap();
		println!("Got new Block {:?}", head);
	}
}
