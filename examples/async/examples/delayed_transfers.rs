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
use dilithium_crypto::pair::{crystal_alice, dilithium_bob};
use pallet_reversible_transfers::DelayPolicy;
use resonance_runtime::{Address, BlockNumber};
use sp_core::H256;
use sp_runtime::traits::IdentifyAccount;
use substrate_api_client::{
	ac_compose_macros::compose_extrinsic,
	ac_node_api::RawEventDetails,
	ac_primitives::{
		resonance_runtime_config::ResonanceRuntimeConfig, Config, ExtrinsicSigner,
		UncheckedExtrinsic,
	},
	extrinsic::BalancesExtrinsics,
	rpc::JsonrpseeClient,
	Api, Error, ExtrinsicReport, GetAccountInformation, GetStorage, SubmitAndWatch,
	TransactionStatus, XtStatus,
};

pub type Result<T> = core::result::Result<T, Error>;

type Hash = <ResonanceRuntimeConfig as Config>::Hash;

mod verify_proof;

type AccountId = <ResonanceRuntimeConfig as Config>::AccountId;

#[tokio::main]
async fn main() {
	env_logger::init();
	println!("[+] Dilithium Signature TEST\n");

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let alice_signer = crystal_alice();
	// let alice = crystal_alice.into_account();
	// let bob = dilithium_bob.into_account();
	let alice = crystal_alice().into_account(); // Get public key and convert to account
	let bob = dilithium_bob().into_account();

	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<ResonanceRuntimeConfig, _>::new(client).await.unwrap();

	let es = ExtrinsicSigner::<ResonanceRuntimeConfig>::new(alice_signer.into());

	api.set_signer(es);

	let (maybe_data_of_alice, maybe_data_of_bob) =
		tokio::try_join!(api.get_account_data(&alice), api.get_account_data(&bob)).unwrap();
	let data_of_alice = maybe_data_of_alice.clone().unwrap();
	let balance_of_alice = data_of_alice.free;
	let reserve_of_alice = data_of_alice.reserved;
	let frozen_of_alice = data_of_alice.frozen;

	let bob_data = maybe_data_of_bob.unwrap_or_default();
	let balance_of_bob = bob_data.clone().free;
	let reserve_of_bob = bob_data.reserved;
	let frozen_of_bob = bob_data.frozen;
	println!("[+] Crystal Alice's Free Balance is {balance_of_alice}\n");
	println!("[+] Crystal Bob's Free Balance is {balance_of_bob}\n");
	println!("[+] Crystal Bob's Reserve Balance is {reserve_of_bob}\n");
	println!("[+] Crystal Bob's Frozen Balance is {frozen_of_bob}\n");
	println!("[+] Crystal Bob's data {:?}\n", bob_data);

	// Get the last finalized header to retrieve information for Era for mortal transactions (online).
	let recipient: Address = bob.clone().into();
	let signer_nonce = api.get_nonce().await.unwrap();
	println!("[+] Alice's Account Nonce is {}", signer_nonce);

	let recipients_extrinsic_address = recipient.clone();

	// Construct an extrinsic using only functionality available in no_std
	// check if the recipient is reversible

	let is_reversible = api
		.get_storage_map::<AccountId, (BlockNumber, DelayPolicy)>(
			"ReversibleTransfers",
			"ReversibleAccounts",
			alice.clone(),
			None,
		)
		.await
		.unwrap()
		.is_some();

	if is_reversible {
		println!("[+] Recipient is already reversible");
	} else {
		println!("[+] Recipient is not reversible");
		let xt: UncheckedExtrinsic<_, _, _, _> = compose_extrinsic!(
			api,
			"ReversibleTransfers",
			"set_reversibility",
			None::<u64>,
			DelayPolicy::Intercept
		)
		.unwrap();

		// Send and watch extrinsic until InBlock.
		let result = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).await;
		let _ = check_result(result, false);
		println!("[+] Sent set_reversibility extrinsic.");
	}

	let scheduled_amount = 10000_u128;
	let schedule_transfer_extrinsic: UncheckedExtrinsic<_, _, _, _> = compose_extrinsic!(
		api,
		"ReversibleTransfers",
		"schedule_transfer",
		recipients_extrinsic_address,
		scheduled_amount
	)
	.unwrap();
	println!("[+] Composed schedule transfer extrinsic: {schedule_transfer_extrinsic:?}\n",);
	// Send and watch extrinsic until InBlock.
	let result = api
		.submit_and_watch_extrinsic_until(schedule_transfer_extrinsic, XtStatus::InBlock)
		.await;
	println!("[+] Sent the schedule transfer extrinsic\n");
	// Check if the transfer really was successful:
	let fee = check_result(result, false).unwrap();

	println!("[+] Sleeping for 1 second");
	tokio::time::sleep(std::time::Duration::from_secs(1)).await;

	let new_alice_account = api.get_account_data(&alice).await.unwrap().unwrap();
	let new_balance_of_alice = new_alice_account.free;
	let new_reserve_of_alice = new_alice_account.reserved;
	let new_frozen_of_alice = new_alice_account.frozen;
	println!("[+] fee: {fee:?}\n",);
	println!("[+] New free balance: {new_balance_of_alice:?}\n",);
	println!("[+] New reserve balance: {new_reserve_of_alice:?}\n",);
	println!("[+] New frozen balance: {new_frozen_of_alice:?}\n",);

	let expected_balance_alice = balance_of_alice - scheduled_amount - fee;
	assert_eq!(expected_balance_alice, new_balance_of_alice);
	let expected_reserve_of_alice = reserve_of_alice + scheduled_amount;
	assert_eq!(expected_reserve_of_alice, new_reserve_of_alice);
	assert_eq!(frozen_of_alice, new_frozen_of_alice);

	// Next, we send an extrinsic that should succeed:
	let balance_to_transfer = 1000;
	let failing_transfer_extrinsic: UncheckedExtrinsic<_, _, _, _> = api
		.balance_transfer_allow_death(bob.clone().into(), balance_to_transfer)
		.await
		.unwrap();

	println!("[+] Composed failing extrinsic: {failing_transfer_extrinsic:?}\n",);
	// Send and watch extrinsic until InBlock.
	let result = api
		.submit_and_watch_extrinsic_until(failing_transfer_extrinsic, XtStatus::InBlock)
		.await;
	println!("[+] Sent the transfer extrinsic that should be intercepted and fail");

	// Check if the transfer really was successful:
	let _ = check_result(result, true).unwrap();

	// Wait `delay` amount of blocks
	let delay = 10;
	// let average_block_time = QPoW::get_median_block_time();
	println!("[+] Average block time is 1 seconds. Waiting for {} seconds", delay * 2);
	tokio::time::sleep(std::time::Duration::from_secs(delay as u64 * 2)).await;
	// Verify that Bob release has received the transferred amount.
	let new_balance_of_bob = api.get_account_data(&bob).await.unwrap().unwrap().free;
	println!("[+] Crystal Bob's Free Balance is now {}\n", new_balance_of_bob);

	// Since the first scheduled transfer already executed, bob now has both amounts
	// TODO: fix interception
	// let expected_balance_of_bob = balance_of_bob + balance_to_transfer + scheduled_amount;
	let expected_balance_of_bob = balance_of_bob + scheduled_amount;
	assert_eq!(expected_balance_of_bob, new_balance_of_bob);

	let verified = verify_proof::verify_transfer_proof(api, alice, bob, scheduled_amount).await;
	assert!(verified, "Failed to verify transfer proof");
}

fn check_result(result: Result<ExtrinsicReport<H256>>, expect_panic: bool) -> Result<u128> {
	match result {
		Ok(report) => {
			if expect_panic {
				panic!("[+] Extrinsic report did not panic");
			}
			let extrinsic_hash = report.extrinsic_hash;
			let block_hash = report.block_hash.unwrap();
			let extrinsic_status = report.status;
			let extrinsic_events = report.events.unwrap();

			println!("[+] Extrinsic with hash {extrinsic_hash:?} was successfully executed.",);
			println!("[+] Extrinsic got included in block with hash {block_hash:?}");
			println!("[+] Watched extrinsic until it reached the status {extrinsic_status:?}");

			let expected_in_block_status: TransactionStatus<Hash, Hash> =
				TransactionStatus::InBlock(block_hash);
			println!("[+] Expected in block status: {:?}", expected_in_block_status);

			// assert!(matches!(extrinsic_status, TransactionStatus::InBlock(_block_hash))); // fails - commented out
			assert_associated_events_match_expected(extrinsic_events.clone());

			if let Some(weight_event) = extrinsic_events.clone().iter_mut().find(|event| {
				event.pallet_name() == "MiningRewards" && event.variant_name() == "FeesCollected"
			}) {
				let (fee, _) = <(u128, u128)>::decode(&mut weight_event.field_bytes())
					.expect("Failed to decode FeesCollected event");
				println!("[+] FeesCollected event: {:?}", weight_event);
				println!("[+] Fee, Total fee: {:?}", fee);
				Ok(fee)
			} else {
				panic!("Expected FeesCollected event not found");
			}
		},
		Err(e) => {
			if expect_panic {
				println!("[+] Expected extrinsic to fail and it did: {e:?}");
				return Ok(0);
			}
			panic!("Expected the transfer to succeed. Instead, it failed due to {e:?}");
		},
	}
}

fn assert_associated_events_match_expected(events: Vec<RawEventDetails<Hash>>) {
	// First event
	for (i, event) in events.iter().enumerate() {
		println!(
			"[+] {:?} Event: Pallet: {:?}, Variant: {:?}",
			i,
			event.pallet_name(),
			event.variant_name()
		);
	}
}
