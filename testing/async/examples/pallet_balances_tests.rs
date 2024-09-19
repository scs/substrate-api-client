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

//! Tests for the pallet balances interface functions.

use codec::Encode;
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	ac_primitives::RococoRuntimeConfig, extrinsic::BalancesExtrinsics, rpc::JsonrpseeClient, Api,
	GetAccountInformation, GetBalance, GetTransactionPayment, SubmitAndWatch, XtStatus,
};

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<RococoRuntimeConfig, _>::new(client).await.unwrap();

	let ed = api.get_existential_deposit().await.unwrap();
	println!("[+] Existential deposit is {}\n", ed);

	let alice = AccountKeyring::Alice.to_account_id();
	let alice_signer = AccountKeyring::Alice.pair();
	api.set_signer(alice_signer.into());
	let balance_of_alice = api.get_account_data(&alice).await.unwrap().unwrap().free;
	println!("[+] Alice's Free Balance is {}\n", balance_of_alice);

	let bob = AccountKeyring::Bob.to_account_id();
	let balance_of_bob = api.get_account_data(&bob).await.unwrap().unwrap_or_default().free;
	println!("[+] Bob's Free Balance is {}\n", balance_of_bob);

	// Rough estimate of fees for three transactions
	let dummy_xt = api
		.balance_transfer_keep_alive(bob.clone().into(), balance_of_alice)
		.await
		.unwrap()
		.encode();
	let transaction_fee =
		api.get_fee_details(&dummy_xt.into(), None).await.unwrap().unwrap().final_fee();
	println!("[+] Transaction Fee is {}\n", transaction_fee);

	let xt = api
		.balance_transfer_keep_alive(
			bob.clone().into(),
			balance_of_alice / 2 - (3 * transaction_fee),
		)
		.await
		.unwrap();
	let report = api.submit_and_watch_extrinsic_until(xt, XtStatus::Finalized).await;
	// This call should succeed as alice has enough money
	assert!(report.is_ok());

	// Alice now has half of her balance plus two transaction fees left
	// (one has been deducted by the transaction above).
	let estimated_balance_of_alice = balance_of_alice / 2 + 2 * transaction_fee;

	let balance_of_alice = api.get_account_data(&alice).await.unwrap().unwrap().free;
	println!("[+] Alice's Free Balance is {}\n", balance_of_alice);
	assert_eq!(balance_of_alice, estimated_balance_of_alice);

	let xt = api
		.balance_transfer_keep_alive(bob.clone().into(), balance_of_alice - transaction_fee - 1)
		.await
		.unwrap();

	let report = api.submit_and_watch_extrinsic_until(xt, XtStatus::Finalized).await;
	// This call should fail as alice would fall below the existential deposit
	assert!(report.is_err());

	let balance_of_alice = api.get_account_data(&alice).await.unwrap().unwrap().free;
	println!("[+] Alice's Free Balance is {}\n", balance_of_alice);

	let dummy_xt = api
		.balance_transfer_allow_death(bob.clone().into(), balance_of_alice)
		.await
		.unwrap()
		.encode();
	let transaction_fee =
		api.get_fee_details(&dummy_xt.into(), None).await.unwrap().unwrap().final_fee();

	let xt = api
		.balance_transfer_allow_death(bob.clone().into(), balance_of_alice - transaction_fee - 1)
		.await
		.unwrap();
	let result = api.submit_and_watch_extrinsic_until(xt, XtStatus::Finalized).await;
	// With allow_death the call should succeed
	assert!(result.is_ok());

	let alice_account = api.get_account_data(&alice).await.unwrap();
	// Alice account should not exist anymore so we excpect an error
	assert!(alice_account.is_none());

	let balance_of_bob = api.get_account_data(&bob).await.unwrap().unwrap_or_default().free;
	println!("[+] Bob's Free Balance is {}\n", balance_of_bob);
}
