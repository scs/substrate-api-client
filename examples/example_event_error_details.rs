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

use clap::{load_yaml, App};
use codec::Decode;
use sp_core::crypto::Pair;
use sp_keyring::AccountKeyring;
use sp_runtime::{app_crypto::sp_core::sr25519, AccountId32 as AccountId, MultiAddress};
use std::sync::mpsc::channel;
use substrate_api_client::{
	rpc::WsRpcClient, Api, ApiResult, AssetTipExtrinsicParams, StaticEvent, XtStatus,
};

// Look at the how the transfer event looks like in in the metadata
#[derive(Decode)]
struct TransferEventArgs {
	_from: AccountId,
	_to: AccountId,
	_value: u128,
}

impl StaticEvent for TransferEventArgs {
	const PALLET: &'static str = "Balances";
	const EVENT: &'static str = "Transfer";
}

fn main() {
	env_logger::init();
	let url = get_node_url_from_cli();

	// initialize api and set the signer (sender) that is used to sign the extrinsics
	let from = AccountKeyring::Alice.pair();

	let client = WsRpcClient::new(&url);
	let api = Api::<sr25519::Pair, _, AssetTipExtrinsicParams>::new(client)
		.map(|api| api.set_signer(from.clone()))
		.unwrap();

	let from_account_id = AccountKeyring::Alice.to_account_id();

	let amount = match api.get_account_data(&from_account_id).unwrap() {
		Some(alice) => {
			println!("[+] Alice's Free Balance is is {}\n", alice.free);
			alice.free
		},
		None => {
			println!("[+] Alice's Free Balance is is 0\n");
			10000000000000000000
		},
	};

	let to = AccountKeyring::Bob.to_account_id();

	let balance_of_bob = match api.get_account_data(&to).unwrap() {
		Some(bob) => bob.free,
		None => 0,
	};

	println!("[+] Bob's Free Balance is {}\n", balance_of_bob);
	// generate extrinsic
	let xt = api.balance_transfer(MultiAddress::Id(to.clone()), amount);

	println!(
		"Sending an extrinsic from Alice (Key = {}),\n\nto Bob (Key = {})\n",
		from.public(),
		to
	);
	println!("[+] Composed extrinsic: {:?}\n", xt);

	// Send and watch extrinsic until Ready.
	let _tx_hash = api.send_extrinsic(xt.hex_encode(), XtStatus::Ready).unwrap();
	println!("[+] Transaction got included into the TxPool.");

	// Transfer should fail as Alice wants to transfer all her balance. She does not have enough money to pay the fees.
	let (events_in, events_out) = channel();
	api.subscribe_events(events_in).unwrap();
	let args: ApiResult<TransferEventArgs> = api.wait_for_event(&events_out);
	match args {
		Ok(_transfer) => {
			panic!("Exptected the call to fail.");
		},
		Err(e) => {
			println!("[+] Couldn't execute the extrinsic due to {:?}\n", e);
			let string_error = format!("{:?}", e);
			assert!(string_error.contains("pallet: \"Balances\""));
			assert!(string_error.contains("error: \"InsufficientBalance\""));
		},
	};

	// Verify that Bob's free Balance hasn't changed.
	let bob = api.get_account_data(&to).unwrap().unwrap();
	println!("[+] Bob's Free Balance is now {}\n", bob.free);
	assert_eq!(balance_of_bob, bob.free);

	// Verify that Alice's free Balance decreased: paid fees.
	let alice = api.get_account_data(&from_account_id).unwrap().unwrap();
	println!("[+] Alice's Free Balance is now {}\n", alice.free);
	assert!(amount > alice.free);
}

pub fn get_node_url_from_cli() -> String {
	let yml = load_yaml!("cli.yml");
	let matches = App::from_yaml(yml).get_matches();

	let node_ip = matches.value_of("node-server").unwrap_or("ws://127.0.0.1");
	let node_port = matches.value_of("node-port").unwrap_or("9944");
	let url = format!("{}:{}", node_ip, node_port);
	println!("Interacting with node on {}\n", url);
	url
}
