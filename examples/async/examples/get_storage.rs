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

use frame_system::AccountInfo as GenericAccountInfo;
use kitchensink_runtime::AccountId;
use pallet_staking::Exposure;
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	ac_primitives::{AssetRuntimeConfig, Config},
	rpc::JsonrpseeClient,
	Api, GetAccountInformation, GetStorage,
};

// To test this example with CI we run it against the Substrate kitchensink node, which uses the asset pallet.
// Therefore, we need to use the `AssetRuntimeConfig` in this example.
// ! However, most Substrate runtimes do not use the asset pallet at all. So if you run an example against your own node
// you most likely should use `DefaultRuntimeConfig` instead.

type AccountInfo = GenericAccountInfo<
	<AssetRuntimeConfig as Config>::Index,
	<AssetRuntimeConfig as Config>::AccountData,
>;

type Balance = <AssetRuntimeConfig as Config>::Balance;

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api.
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).await.unwrap();

	// Get some plain storage values.
	let (maybe_balance, proof) = tokio::try_join!(
		api.get_storage::<Option<Balance>>("Balances", "TotalIssuance", None),
		api.get_storage_value_proof("Balances", "TotalIssuance", None)
	)
	.unwrap();
	println!("[+] TotalIssuance is {:?}", maybe_balance.unwrap());
	println!("[+] StorageValueProof: {:?}", proof);

	// Get the AccountInfo of Alice and the associated StoragePrefix.
	let account: sp_core::sr25519::Public = AccountKeyring::Alice.public();
	let (maybe_account_info, key_prefix) = tokio::try_join!(
		api.get_storage_map::<_, Option<AccountInfo>>("System", "Account", account, None),
		api.get_storage_map_key_prefix("System", "Account")
	)
	.unwrap();

	println!("[+] AccountInfo for Alice is {:?}", maybe_account_info.unwrap());
	println!("[+] Key prefix for System Account map is {:?}", key_prefix);

	// Get Alice's and Bobs AccountNonce with api.get_nonce(). Alice will be set as the signer for
	// the current api, so the nonce retrieval can be simplified:
	let signer = AccountKeyring::Alice.pair();
	api.set_signer(signer.into());
	let bob = AccountKeyring::Bob.to_account_id();

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

	let storage_double_map_key_prefix = api
		.get_storage_double_map_key_prefix("Staking", "ErasStakers", 0)
		.await
		.unwrap();
	let double_map_storage_keys = api
		.get_storage_keys_paged(Some(storage_double_map_key_prefix), max_keys, None, None)
		.await
		.unwrap();

	// Get the storage values that belong to the retrieved storage keys.
	for storage_key in double_map_storage_keys.iter() {
		println!("Retrieving value for key {:?}", storage_key);
		// We're expecting Exposure as return value because we fetch a storage value with prefix combination of "Staking" + "EraStakers" + 0.
		let storage_data: Exposure<AccountId, Balance> =
			api.get_storage_by_key(storage_key.clone(), None).await.unwrap().unwrap();
		println!("Retrieved data {:?}", storage_data);
	}
}
