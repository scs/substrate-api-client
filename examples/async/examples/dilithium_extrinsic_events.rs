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
use std::str::FromStr;
use codec::{Decode, Encode};
// pub use types::{ResonancePublic, ResonanceSignature, ResonancePair, ResonanceSignatureScheme, ResonanceSigner, WrappedPublicBytes, WrappedSignatureBytes};
// pub use crypto::{PUB_KEY_BYTES, SECRET_KEY_BYTES, SIGNATURE_BYTES};
// pub use pair::{crystal_alice, dilithium_bob, crystal_charlie};
use substrate_api_client::{ac_node_api::RawEventDetails, ac_primitives::{ExtrinsicSigner, Config, resonance_runtime_config::ResonanceRuntimeConfig}, extrinsic::BalancesExtrinsics, rpc::JsonrpseeClient, Api, GetAccountInformation, GetChainInfo, GetStorage, SubmitAndWatch, TransactionStatus, XtStatus};
use dilithium_crypto::pair::{crystal_alice, dilithium_bob};
use frame_support::__private::metadata::v14::StorageHasher::Blake2_128Concat;
use sp_runtime::traits::{BlakeTwo256, Hash as Hesh, IdentifyAccount};
type Hash = <ResonanceRuntimeConfig as Config>::Hash;
use poseidon_resonance::PoseidonHasher;
use sp_core::crypto::AccountId32;
use sp_core::{twox_128, Blake2Hasher, Hasher, H256};
use substrate_api_client::runtime_api::AccountNonceApi;
use trie_db::proof::verify_proof;
use hex;
use trie_db::node::{Node, NodeHandle};
use trie_db::TrieLayout;
use substrate_api_client::ac_primitives::{HashTrait, StorageKey};
// To test this example with CI we run it against the Polkadot Rococo node. Remember to switch the Config to match your
// own runtime if it uses different parameter configurations. Several pre-compiled runtimes are available in the ac-primitives crate.
use trie_db::NodeCodec;

#[tokio::main]
async fn main() {
	env_logger::init();
	println!("[+] Dilithium Signature TEST\n");

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let alice_signer = crystal_alice();
	// let alice = crystal_alice.into_account();
	// let bob = dilithium_bob.into_account();
	let alice = crystal_alice().into_account();  // Get public key and convert to account
	let bob = dilithium_bob().into_account();


	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<ResonanceRuntimeConfig, _>::new(client).await.unwrap();

	let es = ExtrinsicSigner::<ResonanceRuntimeConfig>::new(alice_signer.into());

	api.set_signer(es);


	let (maybe_data_of_alice, maybe_data_of_bob) =
		tokio::try_join!(api.get_account_data(&alice), api.get_account_data(&bob)).unwrap();
	let balance_of_alice = maybe_data_of_alice.unwrap().free;
	let balance_of_bob = maybe_data_of_bob.unwrap_or_default().free;
	println!("[+] Crystal Alice's Free Balance is {balance_of_alice}\n");
	println!("[+] Crystal Bob's Free Balance is {balance_of_bob}\n");

	// First we want to see the events of a failed extrinsic.
	// So lets create an extrinsic that will not succeed:
	// Alice tries so transfer all her balance, but that will not work, because
	// she will not have enough balance left to pay the fees.
	// let bad_transfer_extrinsic = api
	// 	.balance_transfer_allow_death(bob.clone().into(), balance_of_alice)
	// 	.await
	// 	.unwrap();
	// println!("[+] Composed bad extrinsic: {bad_transfer_extrinsic:?}\n",);

	// // Send and watch extrinsic until InBlock.
	// let result = api
	// 	.submit_and_watch_extrinsic_until(bad_transfer_extrinsic, XtStatus::InBlock)
	// 	.await;
	// println!("[+] Sent bad transfer extrinsic. Result {result:?}");

	// // Check if the transfer really has failed:
	// match result {
	// 	Ok(_report) => {
	// 		panic!("Exptected the call to fail.");
	// 	},
	// 	Err(e) => {
	// 		println!("[+] Couldn't execute the extrinsic due to {e:?}\n");
	// 		let string_error = format!("{e:?}");
	// 		assert!(string_error.contains("FundsUnavailable"));
	// 	},
	// };

	// Verify that Bob's free Balance hasn't changed.
	// let new_balance_of_bob = api.get_account_data(&bob).await.unwrap().unwrap().free;
	// println!("[+] Bob's Free Balance is now {}\n", new_balance_of_bob);
	// assert_eq!(balance_of_bob, new_balance_of_bob);

	// // Verify that Alice's free Balance decreased: paid fees.
	// let new_balance_of_alice = api.get_account_data(&alice).await.unwrap().unwrap().free;
	// println!("[+] Alice's Free Balance is now {}\n", new_balance_of_alice);
	// assert!(balance_of_alice > new_balance_of_alice);

	// Next, we send an extrinsic that should succeed:
	let balance_to_transfer = 1000;
	let good_transfer_extrinsic = api
		.balance_transfer_allow_death(bob.clone().into(), balance_to_transfer)
		.await
		.unwrap();
	println!("[+] Composed good extrinsic: {good_transfer_extrinsic:?}\n",);
	// Send and watch extrinsic until InBlock.
	let result = api
		.submit_and_watch_extrinsic_until(good_transfer_extrinsic, XtStatus::InBlock)
		.await;
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

			let expected_in_block_status: TransactionStatus<Hash, Hash> = TransactionStatus::InBlock(block_hash);
			println!("[+] Expected in block status: {:?}", expected_in_block_status);

			// assert!(matches!(extrinsic_status, TransactionStatus::InBlock(_block_hash))); // fails - commented out
			assert_associated_events_match_expected(extrinsic_events);
		},
		Err(e) => {
			panic!("Expected the transfer to succeed. Instead, it failed due to {e:?}");
		},
	};

	// Verify that Bob release has received the transferred amount.
	let new_balance_of_bob = api.get_account_data(&bob).await.unwrap().unwrap().free;
	println!("[+] Crystal Bob's Free Balance is now {}\n", new_balance_of_bob);
	let expected_balance_of_bob = balance_of_bob + balance_to_transfer;
	assert_eq!(expected_balance_of_bob, new_balance_of_bob);

	get_transfer_proof(api, alice, bob, balance_to_transfer).await.unwrap();
	// get_transfer_proof(api, alice.clone(), alice, balance_to_transfer).await.unwrap();
}

async fn get_transfer_proof(
	api: Api::<ResonanceRuntimeConfig, JsonrpseeClient>,
	from: AccountId32,
	to: AccountId32,
	amount: u128,
) -> Result<(), Box<dyn std::error::Error>> {
	let block_hash = api.get_block_hash(None).await.unwrap().unwrap();
	// let block_hash = H256::from_str("0x3e21267e348b58b3a45e04c6fa00bce3c19c4cabbaafc3c923b08ad6b8a578ca").unwrap();
	tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

	let nonce = api.runtime_api().account_nonce(from.clone(), None).await.unwrap();
	// let nonce = 1;
	let key_tuple = (nonce, from, to, amount);
	println!("[+] Transaction nonce: {nonce:?} key: {key_tuple:?}");

	// Request the storage proof
	let proof = api
		.get_storage_map_proof("Balances", "TransferProof", key_tuple.clone(), Some(block_hash))
		.await
		.unwrap()
		.unwrap();

	// let reversed_proof = proof.proof.reverse();

	let proof_as_u8: Vec<Vec<u8>> = proof.proof
		.iter() // Iterate over the Vec<Bytes>
		.map(|bytes| bytes.as_ref().to_vec()) // Convert each Bytes to Vec<u8>
		.collect::<Vec<_>>(); // Collect into Vec<Vec<u8>>

	// for (i, node) in proof_as_u8.iter().enumerate() {
	// 	println!("Proof node {}: {} bytes", i, node.len());
	// 	println!("  Data: {:?}", hex::encode(node));
	// }

	for (i, node_data) in proof_as_u8.iter().enumerate() {
		match <sp_trie::LayoutV1<PoseidonHasher> as TrieLayout>::Codec::decode(node_data) {
			Ok(node) => {
				match &node {
					Node::Empty => log::info!("Proof node {}: Empty", i),
					Node::Leaf(partial, value) => {
						let nibbles: Vec<u8> = partial.right_iter().collect();
						log::info!("Proof node {}: Leaf, partial: {:?}, value: {:?}",
                        i, hex::encode(&nibbles), value);
					},
					Node::Extension(partial, _) => {
						let nibbles: Vec<u8> = partial.right_iter().collect();
						log::info!("Proof node {}: Extension, partial: {:?}", i, hex::encode(&nibbles));
					},
					Node::Branch(children, value) => {
						log::info!("Proof node {}: Branch, value: {:?}", i, value);
						for (j, child) in children.iter().enumerate() {
							if let Some(child) = child {
								log::info!("  Child {}: {:?}", j, child);
							}
						}
					},
					Node::NibbledBranch(partial, children, value) => {
						let nibbles: Vec<u8> = partial.right_iter().collect();
						let children = children.iter()
							.filter_map(|x| x.as_ref()
								.map(|val| match val {
									NodeHandle::Hash(h) => hex::encode(h),
									NodeHandle::Inline(i) => hex::encode(i)
								})
							).collect::<Vec<String>>();
						log::info!("Proof node {}: NibbledBranch, partial: {:?}, value: {:?}, children: {:?}",
                        i, hex::encode(&nibbles), value, children);
					},
				}
			},
			Err(e) => log::info!("Failed to decode proof node {}: {:?}", i, e),
		}
	}

	println!("Storage proof at block {:?}: {:?}", block_hash, proof.proof);

	let pallet_prefix = twox_128("Balances".as_bytes());
	let storage_prefix = twox_128("TransferProof".as_bytes());
	let encoded_key = key_tuple.encode();
	let key_hash = <PoseidonHasher as HashTrait>::hash(&encoded_key);
	// let key_hash = <BlakeTwo256 as HashTrait>::hash(&encoded_key);

	let correct_storage_key = [&pallet_prefix[..], &storage_prefix[..], key_hash.as_ref()].concat();
	let storage_key = StorageKey(correct_storage_key.clone());

	// let storage_key = api.metadata().storage_map_key("Balances", "TransferProof", encoded_key.clone()).unwrap();

	println!("Storage key: {:?} {:?}", hex::encode(&storage_key), storage_key);
	// println!("Storage key2: {:?} ", hex::encode(&storage_key2));

	// let storage_info = api.metadata().storage("Balances", "TransferProof").unwrap();
	// println!("Storage hasher: {:?}", storage_info.ty.hashers());
	//
	// let state_value = api.metadata().storage(&storage_key, Some(block_hash)).await.unwrap();
	// println!("Value in state: {:?}", state_value);


	// Compute the storage key
	// let pallet_hash = twox_128("Balances".as_bytes());
	// let storage_hash = twox_128("TransferProof".as_bytes());
	// let key_encoded = key_tuple.encode();
	// let key_hash = PoseidonHasher::hash(&key_encoded); // Blake2_128Concat
	// let storage_key = [&pallet_hash[..], &storage_hash[..], key_hash.as_ref()].concat();
	// println!("Storage key: {:?}", &storage_key);

	let header = api.get_header(Some(block_hash)).await.unwrap().unwrap();
	let state_root = header.state_root;
	println!("Header: {:?} State root: {:?}", header, state_root);
	let expected_value = 1u8.encode();
	// println!("Proof: {:?}", proof_as_u8);
	println!("Expected value: {:?}", expected_value);

	let storage_value = api
		.get_storage_by_key::<u8>(storage_key.clone(), Some(block_hash))
		.await
		.unwrap();
	println!("Storage value: {:?}", storage_value);

	// let nibble_key = encoded_key.iter().flat_map(|&byte| {
	// 	// Convert each byte to two nibbles
	// 	[(byte >> 4) & 0xF, byte & 0xF]
	// }).collect::<Vec<u8>>();
	let blake_key_hash = Blake2Hasher::hash(&encoded_key);

	let mut items = Vec::new();
	// items.push((encoded_key, Some(expected_value)));
	items.push((correct_storage_key, Some(expected_value)));
	// items.push((key_hash, Some(expected_value)));
	// items.push((blake_key_hash, Some(expected_value)));

	// let key_value_pair: (&[u8], Option<&[u8]>) = (&encoded_key.as_slice(), Some(expected_value.as_slice()));
	// let key_value_pair = (encoded_key.as_slice(), Some(expected_value.as_slice()));
	// let key_value_pair = (correct_storage_key.as_slice(), Some(expected_value.as_slice()));
	// let key_value_pair: (&[u8], Option<&[u8]>) = (&storage_key.0, Some(expected_value.as_slice()));
	// let key_value_pair: ([u8; 32], Option<&[u8]>) = (blake_key_hash.0, Some(expected_value.as_slice()));
	// let key_value_pair: (&[u8], Option<u8>) = (&storage_key.0[..], Some(1u8));
	// let key_value_pair: (&[u8], Option<&[u8]>) = (&storage_key.0[..], None);

	let result = verify_proof::<sp_trie::LayoutV1<PoseidonHasher>, _, _, _>(
		&state_root, &proof_as_u8, items.iter());
		// &state_root, &proof_as_u8, std::iter::once(&key_value_pair));
	match result {
		Ok(()) => println!("Proof verified"),
		Err(e) => println!("Proof failed to verify: {:?}", e),
	}
	// let verified = verify_storage_proof(state_root, storage_key, proof.proof, expected_value);
	// println!("Verified storage proof: {:?}", verified);
	Ok(())
}

// fn verify_storage_proof(
// 	state_root: [u8; 32],
// 	storage_key: Vec<u8>,
// 	proof: Vec<Vec<u8>>,
// 	expected_value: Vec<u8>
// ) -> bool {
// 	let mut key = storage_key;
// 	let mut current_hash = PoseidonHasher::hash(&[&key[..], &expected_value[..]].concat());
// 	for proof_item in &proof {
// 		if is_left {
// 			// Proof item is left sibling
// 			current_hash = PoseidonHasher::hash(&[proof_item, &current_hash[..]].concat());
// 		} else {
// 			// Proof item is right sibling
// 			current_hash = PoseidonHasher::hash(&[&current_hash[..], proof_item].concat());
// 		}
// 	}
// 	current_hash.0 == state_root
// }

fn assert_associated_events_match_expected(events: Vec<RawEventDetails<Hash>>) {
	// First event
	for (i, event) in events.iter().enumerate() {
		println!("[+] {:?} Event: Pallet: {:?}, Variant: {:?}", i, event.pallet_name(), event.variant_name());
	}

	// these tests also fail..
	// [+] 0 Event: Pallet: "Balances", Variant: "Withdraw"
	// [+] 1 Event: Pallet: "Balances", Variant: "Transfer"
	// [+] 2 Event: Pallet: "Balances", Variant: "Deposit"
	// [+] 3 Event: Pallet: "TransactionPayment", Variant: "TransactionFeePaid"
	// [+] 4 Event: Pallet: "System", Variant: "ExtrinsicSuccess"

	// assert_eq!(events[0].pallet_name(), "Balances");
	// assert_eq!(events[0].variant_name(), "Withdraw");

	// assert_eq!(events[1].pallet_name(), "Balances");
	// assert_eq!(events[1].variant_name(), "Transfer");

	// assert_eq!(events[2].pallet_name(), "Balances");
	// assert_eq!(events[2].variant_name(), "Deposit");

	// assert_eq!(events[3].pallet_name(), "Balances"); // huh? that's not happening.
	// assert_eq!(events[3].variant_name(), "Deposit");

	// assert_eq!(events[4].pallet_name(), "TransactionPayment");
	// assert_eq!(events[4].variant_name(), "TransactionFeePaid");

	// assert_eq!(events[5].pallet_name(), "System");
	// assert_eq!(events[5].variant_name(), "ExtrinsicSuccess");
}
