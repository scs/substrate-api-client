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
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).unwrap();

	// get some plain storage value
	let result: Balance = api.get_storage("Balances", "TotalIssuance", None).unwrap().unwrap();
	println!("[+] TotalIssuance is {}", result);

	let proof = api.get_storage_value_proof("Balances", "TotalIssuance", None).unwrap();
	println!("[+] StorageValueProof: {:?}", proof);

	let account = AccountKeyring::Alice.public();
	let result: AccountInfo = api
		.get_storage_map("System", "Account", account, None)
		.unwrap()
		.or_else(|| Some(AccountInfo::default()))
		.unwrap();
	println!("[+] AccountInfo for Alice is {:?}", result);

	// get StorageMap key prefix
	let result = api.get_storage_map_key_prefix("System", "Account").unwrap();
	println!("[+] key prefix for System Account map is {:?}", result);

	// get Alice's AccountNonce with api.get_nonce()
	let signer = AccountKeyring::Alice.pair();
	api.set_signer(signer.into());
	println!("[+] Alice's Account Nonce is {}", api.get_nonce().unwrap());

	println!(
		"[+] Bob's Account Nonce is {}",
		api.get_account_nonce(&AccountKeyring::Bob.to_account_id()).unwrap()
	);

	// Get an vector of storage keys, numbering up to the given max keys and that start with the (optionally) given storage key prefix.
	let storage_key_prefix = api.get_storage_map_key_prefix("System", "Account").unwrap();
	let max_keys = 3;
	let storage_keys = api
		.get_storage_keys_paged(Some(storage_key_prefix), max_keys, None, None)
		.unwrap();
	assert_eq!(storage_keys.len() as u32, max_keys);
	// Get the storage values that belong to the retrieved storage keys.
	for storage_key in storage_keys.iter() {
		println!("Retrieving value for key {:?}", storage_key);
		// We're expecting account info as return value because we fetch a storage value with prefix combination of "System" + "Account".
		let storage_data: AccountInfo =
			api.get_storage_by_key(storage_key.clone(), None).unwrap().unwrap();
		println!("Retrieved data {:?}", storage_data);
	}

	let storage_double_map_key_prefix =
		api.get_storage_double_map_key_prefix("Staking", "ErasStakers", 0).unwrap();
	let double_map_storage_keys = api
		.get_storage_keys_paged(Some(storage_double_map_key_prefix), max_keys, None, None)
		.unwrap();

	// Get the storage values that belong to the retrieved storage keys.
	for storage_key in double_map_storage_keys.iter() {
		println!("Retrieving value for key {:?}", storage_key);
		// We're expecting Exposure as return value because we fetch a storage value with prefix combination of "Staking" + "EraStakers" + 0.
		let storage_data: Exposure<AccountId, Balance> =
			api.get_storage_by_key(storage_key.clone(), None).unwrap().unwrap();
		println!("Retrieved data {:?}", storage_data);
	}
}
