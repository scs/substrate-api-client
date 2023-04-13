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
use kitchensink_runtime::{Runtime, Signature};
use sp_keyring::AccountKeyring;
use sp_runtime::{AccountId32 as AccountId, MultiAddress};
use substrate_api_client::{
	ac_node_api::StaticEvent,
	ac_primitives::{AssetTipExtrinsicParams, ExtrinsicSigner},
	extrinsic::BalancesExtrinsics,
	rpc::JsonrpseeClient,
	Api, GetAccountInformation, SubmitAndWatchUntilSuccess,
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

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let alice_signer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().unwrap();
	// ! Careful: AssetTipExtrinsicParams is used here, because the substrate kitchensink runtime uses assets as tips. But for most
	// runtimes, the PlainTipExtrinsicParams needs to be used.
	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(ExtrinsicSigner::<_, Signature, Runtime>::new(alice_signer));

	let alice = AccountKeyring::Alice.to_account_id();
	let balance_of_alice = api.get_account_data(&alice).unwrap().unwrap().free;
	println!("[+] Alice's Free Balance is is {}\n", balance_of_alice);

	let bob = AccountKeyring::Bob.to_account_id();
	let balance_of_bob = api.get_account_data(&bob).unwrap().unwrap_or_default().free;
	println!("[+] Bob's Free Balance is {}\n", balance_of_bob);

	// Generate a transfer extrinsic.
	let xt = api.balance_transfer_allow_death(MultiAddress::Id(bob.clone()), balance_of_alice);
	println!("Sending an extrinsic from Alice (Key = {}),\n\nto Bob (Key = {})\n", alice, bob);
	println!("[+] Composed extrinsic: {:?}\n", xt);

	// Send and watch extrinsic until InBlock.
	let result = api.submit_and_watch_extrinsic_until_success(xt, false);
	println!("[+] Transaction got included into the TxPool.");

	// We expect the transfer to fail as Alice wants to transfer all her balance.
	// Therefore, she will not have enough money to pay the fees.
	match result {
		Ok(_report) => {
			panic!("Exptected the call to fail.");
		},
		Err(e) => {
			println!("[+] Couldn't execute the extrinsic due to {:?}\n", e);
			let string_error = format!("{:?}", e);
			// We expect a TokenError::FundsUnavailable error. See :
			//https://github.com/paritytech/substrate/blob/b42a687c9050cbe04849c45b0c5ccadb82c84948/frame/support/src/traits/tokens/fungible/mod.rs#L177
			assert!(string_error.contains("Other")); //Fixme This is for now not decoded. See issue: #488
		},
	};

	// Verify that Bob's free Balance hasn't changed.
	let new_balance_of_bob = api.get_account_data(&bob).unwrap().unwrap().free;
	println!("[+] Bob's Free Balance is now {}\n", new_balance_of_bob);
	assert_eq!(balance_of_bob, new_balance_of_bob);

	// Verify that Alice's free Balance decreased: paid fees.
	let new_balance_of_alice = api.get_account_data(&alice).unwrap().unwrap().free;
	println!("[+] Alice's Free Balance is now {}\n", new_balance_of_alice);
	assert!(balance_of_alice > new_balance_of_alice);
}
