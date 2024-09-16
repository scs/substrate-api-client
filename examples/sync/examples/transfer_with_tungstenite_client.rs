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

use sp_core::{
	crypto::{Pair, Ss58Codec},
	sr25519,
};
use sp_runtime::MultiAddress;
use substrate_api_client::{
	ac_primitives::RococoRuntimeConfig, extrinsic::BalancesExtrinsics, rpc::TungsteniteRpcClient,
	Api, GetAccountInformation, SubmitAndWatch, XtStatus,
};

// To test this example with CI we run it against the Polkadot Rococo node. Remember to switch the Config to match your
// own runtime if it uses different parameter configurations. Several pre-compiled runtimes are available in the ac-primitives crate.

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
	let mut api = Api::<RococoRuntimeConfig, _>::new(client).unwrap();
	api.set_signer(alice.clone().into());

	// Retrieve bobs current balance.
	let bob = sr25519::Public::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")
		.unwrap();

	let bob_balance = api.get_account_data(&bob.into()).unwrap().unwrap_or_default().free;
	println!("[+] Bob's Free Balance is {}\n", bob_balance);

	// We first generate an extrinsic that will fail to be executed due to missing funds.
	let xt = api
		.balance_transfer_allow_death(MultiAddress::Id(bob.into()), bob_balance + 1)
		.unwrap();
	println!(
		"Sending an extrinsic from Alice (Key = {}),\n\nto Bob (Key = {})\n",
		alice.public(),
		bob
	);
	println!("[+] Composed extrinsic: {:?}\n", xt);

	// Send and watch extrinsic until it fails onchain.
	let result = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock);
	assert!(result.is_err());
	println!("[+] Extrinsic did not get included due to: {:?}\n", result);

	// This time, we generate an extrinsic that will succeed.
	let xt = api
		.balance_transfer_allow_death(MultiAddress::Id(bob.into()), bob_balance / 2)
		.unwrap();
	println!(
		"Sending an extrinsic from Alice (Key = {}),\n\nto Bob (Key = {})\n",
		alice.public(),
		bob
	);
	println!("[+] Composed extrinsic: {:?}\n", xt);

	// Send and watch extrinsic until in block.
	let block_hash = api
		.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock)
		.unwrap()
		.block_hash
		.unwrap();
	println!("[+] Extrinsic got included. Block Hash: {:?}\n", block_hash);

	// Verify that Bob's free Balance increased.
	let bob_new_balance = api.get_account_data(&bob.into()).unwrap().unwrap().free;
	println!("[+] Bob's Free Balance is now {}\n", bob_new_balance);
	assert!(bob_new_balance > bob_balance);
}
