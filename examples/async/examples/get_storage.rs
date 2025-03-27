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

//! Very simple example that shows how to get some simple storage values.

use codec::Encode;
use dilithium_crypto::{crystal_alice, dilithium_bob};
use frame_system::AccountInfo as GenericAccountInfo;
use pallet_recovery::ActiveRecovery;
use sp_keyring::Sr25519Keyring;
use substrate_api_client::{
	ac_compose_macros::compose_extrinsic,
	ac_primitives::{Config, ResonanceRuntimeConfig},
	rpc::JsonrpseeClient,
	Api, GetAccountInformation, GetStorage, SubmitAndWatch, XtStatus,
};

use sp_runtime::traits::IdentifyAccount;

// To test this example with CI we run it against the Polkadot node. Remember to switch the Config to match your
// own runtime if it uses different parameter configurations. Several pre-compiled runtimes are available in the ac-primitives crate.

type AccountInfo = GenericAccountInfo<
	<ResonanceRuntimeConfig as Config>::Index,
	<ResonanceRuntimeConfig as Config>::AccountData,
>;

type Balance = <ResonanceRuntimeConfig as Config>::Balance;
type AccountId = <ResonanceRuntimeConfig as Config>::AccountId;
type BlockNumber = <ResonanceRuntimeConfig as Config>::BlockNumber;
type Friends = Vec<AccountId>;
type Address = <ResonanceRuntimeConfig as Config>::Address;

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api.
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<ResonanceRuntimeConfig, _>::new(client).await.unwrap();

	// Get some plain storage values.
	let (balance, proof) = tokio::try_join!(
		api.get_storage::<Balance>("Balances", "TotalIssuance", None),
		api.get_storage_value_proof("Balances", "TotalIssuance", None)
	)
	.unwrap();
	println!("[+] TotalIssuance is {:?}", balance.unwrap());
	println!("[+] StorageValueProof: {:?}", proof);

	// Get the AccountInfo of Alice and the associated StoragePrefix.
	let account: sp_core::sr25519::Public = Sr25519Keyring::Alice.public();
	let (maybe_account_info, key_prefix) = tokio::try_join!(
		api.get_storage_map::<_, AccountInfo>("System", "Account", account, None),
		api.get_storage_map_key_prefix("System", "Account")
	)
	.unwrap();

	println!("[+] AccountInfo for Alice is {:?}", maybe_account_info.unwrap());
	println!("[+] Key prefix for System Account map is {:?}", key_prefix);

	// Get Alice's and Bobs AccountNonce with api.get_nonce(). Alice will be set as the signer for
	// the current api, so the nonce retrieval can be simplified:
	let signer = crystal_alice();
	api.set_signer(signer.into());
	let bob = dilithium_bob().into_account();

	let (alice_nonce, bob_nonce) =
		tokio::try_join!(api.get_nonce(), api.get_account_nonce(&bob)).unwrap();
	println!("[+] Alice's Account Nonce is {}", alice_nonce);
	println!("[+] Bob's Account Nonce is {}", bob_nonce);

	// Get an vector of storage keys, numbering up to the given max keys and that start with the (optionally) given storage key prefix.
	let storage_key_prefix = api.get_storage_map_key_prefix("System", "Account").await.unwrap();
	let max_keys = 3;
	let storage_keys = api
		.get_storage_keys_paged(Some(storage_key_prefix), max_keys, None, None)
		.await
		.unwrap();
	assert_eq!(storage_keys.len() as u32, max_keys);
	// Get the storage values that belong to the retrieved storage keys.
	for storage_key in storage_keys.iter() {
		println!("Retrieving value for key {:?}", storage_key);
		// We're expecting account info as return value because we fetch a storage value with prefix combination of "System" + "Account".
		let storage_data: AccountInfo =
			api.get_storage_by_key(storage_key.clone(), None).await.unwrap().unwrap();
		println!("Retrieved data {:?}", storage_data);
	}

	// Create a recovery, so we can fetch an actual ActiveRecovery state from the chain.
	// NOTE: Disabled because we don't have recovery pallet. We should have it though. 

	// let alice = Sr25519Keyring::Alice.to_account_id();
	// let alice_multiaddress: Address = alice.clone().into();
	// let charlie = Sr25519Keyring::Charlie.to_account_id();
	// let threshold: u16 = 2;
	// let delay_period: u32 = 1000;

	// let xt = compose_extrinsic!(
	// 	&api,
	// 	"Recovery",
	// 	"create_recovery",
	// 	vec![bob, charlie],
	// 	threshold,
	// 	delay_period
	// )
	// .unwrap();

	// let _report = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).await.unwrap();

	// // Set Bob as signer, so we can send the recevory extrinsic as Bob.
	// let signer2 = Sr25519Keyring::Bob.pair();
	// api.set_signer(signer2.into());
	// let xt = compose_extrinsic!(&api, "Recovery", "initiate_recovery", alice_multiaddress).unwrap();

	// println!("{:?}", xt.encode());
	// let _report = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).await.unwrap();

	// let storage_double_map_key_prefix = api
	// 	.get_storage_double_map_key_prefix("Recovery", "ActiveRecoveries", &alice)
	// 	.await
	// 	.unwrap();
	// let double_map_storage_keys = api
	// 	.get_storage_keys_paged(Some(storage_double_map_key_prefix), max_keys, None, None)
	// 	.await
	// 	.unwrap();

	// // Get the storage values that belong to the retrieved storage keys.
	// for storage_key in double_map_storage_keys.iter() {
	// 	println!("Retrieving value for key {:?}", storage_key);
	// 	// We're expecting Exposure as return value because we fetch a storage value with prefix combination of "Staking" + "EraStakers" + 0.
	// 	let storage_data: ActiveRecovery<BlockNumber, Balance, Friends> =
	// 		api.get_storage_by_key(storage_key.clone(), None).await.unwrap().unwrap();
	// 	println!("Retrieved data {:?}", storage_data);
	// }
}
