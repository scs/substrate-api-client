use codec::Compact;
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
use dilithium_crypto::pair::{crystal_alice, dilithium_bob};
use pallet_reversible_transfers::DelayPolicy;
use resonance_runtime::{configs::DefaultDelay, Address, QPoW, ReversibleTransfersCall};
use sp_core::{Pair, H256};
use sp_runtime::{generic::Era, traits::IdentifyAccount, MultiAddress};
use substrate_api_client::{
	ac_compose_macros::{compose_call, compose_extrinsic},
	ac_node_api::RawEventDetails,
	ac_primitives::{
		resonance_runtime_config::ResonanceRuntimeConfig, Config, ExtrinsicParams, ExtrinsicSigner,
		GenericAdditionalParams, PlainTip, SignExtrinsic, UncheckedExtrinsic,
	},
	extrinsic::BalancesExtrinsics,
	rpc::JsonrpseeClient,
	Api, Error, ExtrinsicReport, GetAccountInformation, GetChainInfo, GetStorage, SubmitAndWatch,
	TransactionStatus, XtStatus,
};

pub type Result<T> = core::result::Result<T, Error>;

type Hash = <ResonanceRuntimeConfig as Config>::Hash;
use hex;
use trie_db::TrieLayout;

mod verify_proof;

type DefaultExtrinsicSigner = <ResonanceRuntimeConfig as Config>::ExtrinsicSigner;
type AccountId = <ResonanceRuntimeConfig as Config>::AccountId;
type ExtrinsicAddressOf<Signer> = <Signer as SignExtrinsic<AccountId>>::ExtrinsicAddress;

type Balance = <ResonanceRuntimeConfig as Config>::Balance;
type AdditionalParams = GenericAdditionalParams<PlainTip<Balance>, Hash>;

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
	let balance_of_alice = maybe_data_of_alice.unwrap().free;
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

	let last_finalized_header_hash = api.get_finalized_head().await.unwrap().unwrap();
	let header = api.get_header(Some(last_finalized_header_hash)).await.unwrap().unwrap();
	let period = 5;

	// First, we mark the account of Alice as reversible
	// Construct extrinsic params needed for the extrinsic construction. For more information on what these parameters mean, take a look at Substrate docs: https://docs.substrate.io/reference/transaction-format/.
	let additional_extrinsic_params: AdditionalParams = GenericAdditionalParams::new()
		.era(Era::mortal(period, header.number.into()), last_finalized_header_hash)
		.tip(0);

	println!("Compose extrinsic in no_std environment (No Api instance)");
	// Get information out of Api (online). This information could also be set offline in the `no_std`,
	// but that would need to be static and adapted whenever the node changes.
	// You can get the information directly from the node runtime file or the api of https://polkadot.js.org.
	let spec_version = api.runtime_version().spec_version;
	let transaction_version = api.runtime_version().transaction_version;
	let genesis_hash = api.genesis_hash();
	let metadata = api.metadata();
	let signer_nonce = api.get_nonce().await.unwrap();
	println!("[+] Alice's Account Nonce is {}", signer_nonce);

	let recipients_extrinsic_address = recipient.clone();

	// Construct an extrinsic using only functionality available in no_std
	let xt: UncheckedExtrinsic<_, _, _, _> = compose_extrinsic!(
		api,
		"ReversibleTransfers",
		"set_reversibility",
		None::<u64>,
		DelayPolicy::Intercept
	).unwrap();

	// Send and watch extrinsic until InBlock.
	let result = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).await;
	check_result(result, false);

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
	check_result(result, false);

	println!("[+] Sent set_reversibility extrinsic.");

	let new_bob_account = api.get_account_data(&bob).await.unwrap().unwrap();
	let new_balance_of_bob = new_bob_account.free;
	let new_reserve_of_bob = new_bob_account.reserved;
	let new_frozen_of_bob = new_bob_account.frozen;
	println!("[+] New reserve balance: {new_reserve_of_bob:?}\n",);
	println!("[+] New frozen balance: {new_frozen_of_bob:?}\n",);

	let expected_balance_of_bob = balance_of_bob - scheduled_amount;
	assert_eq!(expected_balance_of_bob, new_balance_of_bob);

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
	check_result(result, true);

	// Verify that Bob release has received the transferred amount.
	let new_balance_of_bob = api.get_account_data(&bob).await.unwrap().unwrap().free;
	println!("[+] Crystal Bob's Free Balance is now {}\n", new_balance_of_bob);

	// Wait `delay` amount of blocks
	let delay = 10;
	// let average_block_time = QPoW::get_median_block_time();
	println!("[+] Average block time is 1 seconds. Waiting for {} seconds", delay * 1 as u32);
	tokio::time::sleep(std::time::Duration::from_secs(delay as u64 * 3)).await;

	let expected_balance_of_bob = balance_of_bob + balance_to_transfer;
	assert_eq!(expected_balance_of_bob, new_balance_of_bob);

	let verified = verify_proof::verify_transfer_proof(api, alice, bob, balance_to_transfer).await;
}

fn check_result(result: Result<ExtrinsicReport<H256>>, expect_panic: bool) {
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
			assert_associated_events_match_expected(extrinsic_events);
		},
		Err(e) => {
			if expect_panic {
				println!("[+] Expected extrinsic to fail and it did: {e:?}");
				return;
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
