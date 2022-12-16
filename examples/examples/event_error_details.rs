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

use codec::Decode;
use kitchensink_runtime::Runtime;
use sp_core::crypto::Pair;
use sp_keyring::AccountKeyring;
use sp_runtime::{AccountId32 as AccountId, MultiAddress};
use substrate_api_client::{
	rpc::JsonrpseeClient, Api, AssetTipExtrinsicParams, GetAccountInformation, Result, StaticEvent,
	SubmitAndWatch, SubscribeEvents, SubscribeFrameSystem, XtStatus,
};

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

#[tokio::main]
async fn main() {
	env_logger::init();

	// initialize api and set the signer (sender) that is used to sign the extrinsics
	let from = AccountKeyring::Alice.pair();

	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(from.clone());

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
	let _tx_hash = api.submit_and_watch_extrinsic_until(&xt.hex_encode(), XtStatus::Ready).unwrap();
	println!("[+] Transaction got included into the TxPool.");

	// Transfer should fail as Alice wants to transfer all her balance. She does not have enough money to pay the fees.
	let mut subscription = api.subscribe_system_events().unwrap();
	let args: Result<TransferEventArgs> = api.wait_for_event(&mut subscription);
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
