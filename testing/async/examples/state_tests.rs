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

//! Tests for the state rpc interface functions.

use codec::Decode;
use pallet_balances::AccountData as GenericAccountData;
use pallet_society::Vote;
use sp_core::{crypto::Ss58Codec, sr25519};
use sp_keyring::Sr25519Keyring;
use substrate_api_client::{
	ac_primitives::{Config, RococoRuntimeConfig},
	rpc::JsonrpseeClient,
	Api, GetChainInfo, GetStorage,
};

type KitchensinkConfig = RococoRuntimeConfig;
type Balance = <KitchensinkConfig as Config>::Balance;
type AccountData = GenericAccountData<Balance>;

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let api = Api::<KitchensinkConfig, _>::new(client).await.unwrap();

	let alice = Sr25519Keyring::Alice.to_account_id();
	let block_hash = api.get_block_hash(None).await.unwrap().unwrap();
	let alice_stash =
		sr25519::Public::from_ss58check("5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY")
			.unwrap();

	// Tests
	let _total_issuance: Balance =
		api.get_storage("Balances", "TotalIssuance", None).await.unwrap().unwrap();
	let _total_issuance: Balance = api
		.get_storage("Balances", "TotalIssuance", Some(block_hash))
		.await
		.unwrap()
		.unwrap();
	let _account_info: AccountData =
		api.get_storage_map("System", "Account", &alice, None).await.unwrap().unwrap();

	let votes: Option<Vote> = api
		.get_storage_double_map("Society", "DefenderVotes", 0, alice_stash, None)
		.await
		.unwrap();
	assert!(votes.is_none());

	// Ensure the prefix matches the actual storage key:
	let storage_key_prefix = api.get_storage_map_key_prefix("System", "Account").await.unwrap();
	let storage_key = api.metadata().storage_map_key("System", "Account", &alice).unwrap();

	let prefix_len = storage_key_prefix.0.len();
	assert_eq!(storage_key_prefix.0, storage_key.0[..prefix_len]);

	let _account_data: AccountData =
		api.get_storage_by_key(storage_key.clone(), None).await.unwrap().unwrap();
	let account_data_opaque =
		api.get_opaque_storage_by_key(storage_key.clone(), None).await.unwrap().unwrap();
	let _account_data = AccountData::decode(&mut account_data_opaque.as_slice()).unwrap();
	let _value_proof = api
		.get_storage_value_proof("Balances", "TotalIssuance", None)
		.await
		.unwrap()
		.unwrap();
	let _map_proof = api
		.get_storage_map_proof("System", "Account", &alice, None)
		.await
		.unwrap()
		.unwrap();
	let _double_map_proof = api
		.get_storage_double_map_proof("Society", "DefenderVotes", 0, &alice, None)
		.await
		.unwrap()
		.unwrap();
	let _storage_proof = api
		.get_storage_proof_by_keys(vec![storage_key.clone()], None)
		.await
		.unwrap()
		.unwrap();
	let _keys = api.get_keys(storage_key, None).await.unwrap().unwrap();
	let _constants: Balance = api.get_constant("Balances", "ExistentialDeposit").await.unwrap();

	let max_keys = 2003;
	let result = api
		.get_storage_keys_paged_limited(Some(storage_key_prefix.clone()), max_keys, None, None)
		.await;
	assert!(result.is_err());
	assert!(format!("{result:?}").contains("count exceeds maximum value"));

	let storage_keys = api
		.get_storage_keys_paged(Some(storage_key_prefix), max_keys, None, None)
		.await
		.unwrap();
	assert_eq!(storage_keys.len() as u32, 13);

	let max_keys = 20;
	let storage_keys =
		api.get_storage_keys_paged_limited(None, max_keys, None, None).await.unwrap();
	assert_eq!(storage_keys.len() as u32, max_keys);

	let storage_keys = api.get_storage_keys_paged(None, max_keys, None, None).await.unwrap();
	assert_eq!(storage_keys.len() as u32, max_keys);
}
