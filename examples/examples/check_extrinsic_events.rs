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

use sp_keyring::AccountKeyring;
use substrate_api_client::{
	ac_node_api::EventDetails,
	ac_primitives::{AssetRuntimeConfig, Config, ExtrinsicSigner},
	extrinsic::BalancesExtrinsics,
	rpc::JsonrpseeClient,
	Api, GetAccountInformation, SubmitAndWatch, TransactionStatus, XtStatus,
};

// To test this example with CI we run it against the Substrate kitchensink node, which uses the asset pallet.
// Therefore, we need to use the `AssetRuntimeConfig` in this example.
// ! However, most Substrate runtimes do not use the asset pallet at all. So if you run an example against your own node
// you most likely should use `DefaultRuntimeConfig` instead.

type Hash = <AssetRuntimeConfig as Config>::Hash;

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let alice_signer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).unwrap();
	api.set_signer(ExtrinsicSigner::<AssetRuntimeConfig>::new(alice_signer));

	let alice = AccountKeyring::Alice.to_account_id();
	let balance_of_alice = api.get_account_data(&alice).unwrap().unwrap().free;
	println!("[+] Alice's Free Balance is {balance_of_alice}\n");

	let bob = AccountKeyring::Bob.to_account_id();
	let balance_of_bob = api.get_account_data(&bob).unwrap().unwrap_or_default().free;
	println!("[+] Bob's Free Balance is {balance_of_bob}\n");

	// First we want to see the events of a failed extrinsic.
	// So lets create an extrinsic that will not succeed:
	// Alice tries so transfer all her balance, but that will not work, because
	// she will not have enough balance left to pay the fees.
	let bad_transfer_extrinsic =
		api.balance_transfer_allow_death(bob.clone().into(), balance_of_alice);
	println!("[+] Composed extrinsic: {bad_transfer_extrinsic:?}\n",);

	// Send and watch extrinsic until InBlock.
	let result = api.submit_and_watch_extrinsic_until(bad_transfer_extrinsic, XtStatus::InBlock);
	println!("[+] Sent the transfer extrinsic. Result {result:?}");

	// Check if the transfer really has failed:
	match result {
		Ok(_report) => {
			panic!("Exptected the call to fail.");
		},
		Err(e) => {
			println!("[+] Couldn't execute the extrinsic due to {e:?}\n");
			let string_error = format!("{e:?}");
			assert!(string_error.contains("FundsUnavailable"));
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

	// Next, we send an extrinsic that should succeed:
	let balance_to_transfer = 1000;
	let good_transfer_extrinsic =
		api.balance_transfer_allow_death(bob.clone().into(), balance_to_transfer);
	// Send and watch extrinsic until InBlock.
	let result = api.submit_and_watch_extrinsic_until(good_transfer_extrinsic, XtStatus::InBlock);
	println!("[+] Sent the transfer extrinsic.");

	// Check if the transfer really was successful:
	match result {
		Ok(report) => {
			let extrinsic_hash = report.extrinsic_hash;
			let block_hash = report.block_hash.unwrap();
			let extrinsic_status = report.status;
			let extrinsic_events = report.events.unwrap();

			println!("[+] Extrinsic with hash {extrinsic_hash:?} was successfully executed.",);
			println!("[+] Extrinsic got included in block with hash {block_hash:?}");
			println!("[+] Watched extrinsic until it reached the status {extrinsic_status:?}");
			println!("[+] The following events were thrown when the extrinsic was executed: {extrinsic_events:?}");

			assert!(matches!(extrinsic_status, TransactionStatus::InBlock(_block_hash)));
			assert_assosciated_events_match_expected(extrinsic_events);
		},
		Err(e) => {
			panic!("Expected the transfer to succeed. Instead, it failed due to {e:?}");
		},
	};

	// Verify that Bob release has received the transferred amount.
	let new_balance_of_bob = api.get_account_data(&bob).unwrap().unwrap().free;
	println!("[+] Bob's Free Balance is now {}\n", new_balance_of_bob);
	let expected_balance_of_bob = balance_of_bob + balance_to_transfer;
	assert_eq!(expected_balance_of_bob, new_balance_of_bob);
}

fn assert_assosciated_events_match_expected(events: Vec<EventDetails<Hash>>) {
	// First event
	assert_eq!(events[0].pallet_name(), "Balances");
	assert_eq!(events[0].variant_name(), "Withdraw");

	assert_eq!(events[1].pallet_name(), "Balances");
	assert_eq!(events[1].variant_name(), "Transfer");

	assert_eq!(events[2].pallet_name(), "Balances");
	assert_eq!(events[2].variant_name(), "Deposit");

	assert_eq!(events[3].pallet_name(), "Treasury");
	assert_eq!(events[3].variant_name(), "Deposit");

	assert_eq!(events[4].pallet_name(), "Balances");
	assert_eq!(events[4].variant_name(), "Deposit");

	assert_eq!(events[5].pallet_name(), "TransactionPayment");
	assert_eq!(events[5].variant_name(), "TransactionFeePaid");

	assert_eq!(events[6].pallet_name(), "System");
	assert_eq!(events[6].variant_name(), "ExtrinsicSuccess");
}
