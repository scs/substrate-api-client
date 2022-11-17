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

//! This examples shows how to use the compose_extrinsic_offline macro which generates an extrinsic
//! without asking the node for nonce and does not need to know the metadata

use ac_primitives::AssetTipExtrinsicParamsBuilder;
use clap::{load_yaml, App};
use kitchensink_runtime::{BalancesCall, Header, RuntimeCall};
use sp_keyring::AccountKeyring;
use sp_runtime::{generic::Era, MultiAddress};
use substrate_api_client::{
	compose_extrinsic_offline, rpc::WsRpcClient, Api, AssetTipExtrinsicParams,
	UncheckedExtrinsicV4, XtStatus,
};

fn main() {
	env_logger::init();
	let url = get_node_url_from_cli();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let from = AccountKeyring::Alice.pair();
	let client = WsRpcClient::new(&url);

	let api = Api::<_, _, AssetTipExtrinsicParams>::new(client)
		.map(|api| api.set_signer(from))
		.unwrap();

	// Information for Era for mortal transactions.
	let head = api.get_finalized_head().unwrap().unwrap();
	let h: Header = api.get_header(Some(head)).unwrap().unwrap();
	let period = 5;
	let tx_params = AssetTipExtrinsicParamsBuilder::new()
		.era(Era::mortal(period, h.number.into()), head)
		.tip(0);

	let updated_api = api.set_extrinsic_params_builder(tx_params);

	// Get the nonce of Alice.
	let alice_nonce = updated_api.get_nonce().unwrap();
	println!("[+] Alice's Account Nonce is {}\n", alice_nonce);

	// Define the recipient.
	let to = MultiAddress::Id(AccountKeyring::Bob.to_account_id());

	// Create an extrinsic that should get included in the future pool due to a nonce that is too high.
	#[allow(clippy::redundant_clone)]
	let xt: UncheckedExtrinsicV4<_, _> = compose_extrinsic_offline!(
		updated_api.clone().signer.unwrap(),
		RuntimeCall::Balances(BalancesCall::transfer { dest: to.clone(), value: 42 }),
		updated_api.extrinsic_params(alice_nonce + 1)
	);

	println!("[+] Composed Extrinsic:\n {:?}\n", xt);

	// Send and watch extrinsic until InBlock.
	match updated_api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock) {
		Err(error) => {
			println!("Retrieved error {:?}", error);
			assert!(format!("{:?}", error).contains("Future"));
		},
		_ => panic!("Expected an error upon a future extrinsic"),
	}
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
