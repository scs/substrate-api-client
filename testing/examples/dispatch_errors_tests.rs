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

//! Tests for the dispatch error.

use sp_keyring::AccountKeyring;
use sp_runtime::MultiAddress;
use substrate_api_client::{
	ac_primitives::{AssetRuntimeConfig, ExtrinsicSigner},
	extrinsic::BalancesExtrinsics,
	rpc::JsonrpseeClient,
	Api, GetAccountInformation, SubmitAndWatchUntilSuccess,
};

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().unwrap();
	let alice_signer = AccountKeyring::Alice.pair();
	let bob_signer = AccountKeyring::Bob.pair();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).unwrap();

	let alice = AccountKeyring::Alice.to_account_id();
	let balance_of_alice = api.get_account_data(&alice).unwrap().unwrap().free;
	println!("[+] Alice's Free Balance is is {}\n", balance_of_alice);

	let bob = AccountKeyring::Bob.to_account_id();
	let balance_of_bob = api.get_account_data(&bob).unwrap().unwrap_or_default().free;
	println!("[+] Bob's Free Balance is {}\n", balance_of_bob);

	let one = AccountKeyring::One.to_account_id();
	let balance_of_one = api.get_account_data(&one).unwrap().unwrap_or_default().free;
	println!("[+] One's Free Balance is {}\n", balance_of_one);

	//BadOrigin
	api.set_signer(ExtrinsicSigner::<AssetRuntimeConfig>::new(bob_signer));
	//Can only be called by root
	let xt = api.balance_force_set_balance(MultiAddress::Id(alice.clone()), 10);

	let result = api.submit_and_watch_extrinsic_until_success(xt, false);
	assert!(result.is_err());
	assert!(format!("{result:?}").contains("BadOrigin"));
	println!("[+] BadOrigin error: Bob can't force set balance");

	//BelowMinimum
	api.set_signer(ExtrinsicSigner::<AssetRuntimeConfig>::new(alice_signer));
	let xt = api.balance_transfer_allow_death(MultiAddress::Id(one.clone()), 999999);
	let result = api.submit_and_watch_extrinsic_until_success(xt, false);
	assert!(result.is_err());
	assert!(format!("{result:?}").contains("(BelowMinimum"));
	println!("[+] BelowMinimum error: balance (999999) is below the existential deposit");
}
