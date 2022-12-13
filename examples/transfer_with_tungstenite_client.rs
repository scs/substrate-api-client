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

//! Very simple example that shows how to use a predefined extrinsic from the extrinsic module.

use kitchensink_runtime::Runtime;
use sp_core::{
	crypto::{Pair, Ss58Codec},
	sr25519,
};
use sp_runtime::MultiAddress;
use substrate_api_client::{
	rpc::TungsteniteRpcClient, Api, AssetTipExtrinsicParams, GetAccountInformation, SubmitAndWatch,
	XtStatus,
};

fn main() {
	env_logger::init();
	// Alice's seed: subkey inspect //Alice.
	let alice: sr25519::Pair = Pair::from_string(
		"0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
		None,
	)
	.unwrap();
	println!("signer account: {}", alice.public().to_ss58check());

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let client = TungsteniteRpcClient::with_default_url(100);

	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(alice.clone());

	// Bob
	let bob = sr25519::Public::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")
		.unwrap();

	match api.get_account_data(&bob.into()).unwrap() {
		Some(account_data) => println!("[+] Bob's Free Balance is is {}\n", account_data.free),
		None => println!("[+] Bob's Free Balance is is 0\n"),
	}
	// Generate extrinsic.
	let xt = api.balance_transfer(MultiAddress::Id(bob.into()), 1000000000000);

	println!(
		"Sending an extrinsic from Alice (Key = {}),\n\nto Bob (Key = {})\n",
		alice.public(),
		bob
	);

	println!("[+] Composed extrinsic: {:?}\n", xt);

	// Send and watch extrinsic until in block.
	let block_hash = api
		.submit_and_watch_extrinsic_until(&xt.hex_encode(), XtStatus::InBlock)
		.unwrap();
	println!("[+] Transaction got included. Hash: {:?}\n", block_hash);

	// Verify that Bob's free Balance increased.
	let bob_account_data = api.get_account_data(&bob.into()).unwrap().unwrap();
	println!("[+] Bob's Free Balance is now {}\n", bob_account_data.free);
}
