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
use kitchensink_runtime::Runtime;
use pallet_balances::AccountData as GenericAccountData;
use pallet_staking::Exposure;
use sp_core::{crypto::Ss58Codec, sr25519};
use sp_keyring::AccountKeyring;
use sp_staking::EraIndex;
use substrate_api_client::{
	rpc::JsonrpseeClient, Api, AssetTipExtrinsicParams, GetBlock, GetStorage,
};

type Balance = <Runtime as pallet_balances::Config>::Balance;
type AccountData = GenericAccountData<Balance>;
type ErasStakers = Exposure<
	<Runtime as frame_system::Config>::AccountId,
	<Runtime as pallet_staking::Config>::CurrencyBalance,
>;

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().unwrap();
	let alice_pair = AccountKeyring::Alice.pair();
	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(alice_pair);

	let alice = AccountKeyring::Alice.to_account_id();
	let block_hash = api.get_block_hash(None).unwrap().unwrap();
	let alice_stash =
		sr25519::Public::from_ss58check("5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY")
			.unwrap();

	// Tests
	let _total_issuance: Balance =
		api.get_storage_value("Balances", "TotalIssuance", None).unwrap().unwrap();
	let _total_issuance: Balance = api
		.get_storage_value("Balances", "TotalIssuance", Some(block_hash))
		.unwrap()
		.unwrap();
	let _account_info: AccountData =
		api.get_storage_map("System", "Account", &alice, None).unwrap().unwrap();
	let _era_stakers: ErasStakers = api
		.get_storage_double_map("Staking", "ErasStakers", EraIndex::default(), alice_stash, None)
		.unwrap()
		.unwrap();

	// Ensure the prefix matches the actual storage key:
	let storage_key_prefix = api.get_storage_map_key_prefix("System", "Account").unwrap();
	let storage_key = api.metadata().storage_map_key("System", "Account", &alice).unwrap();

	let prefix_len = storage_key_prefix.0.len();
	assert_eq!(storage_key_prefix.0, storage_key.0[..prefix_len]);

	let _account_data: AccountData =
		api.get_storage_by_key_hash(storage_key.clone(), None).unwrap().unwrap();
	let account_data_opaque =
		api.get_opaque_storage_by_key_hash(storage_key.clone(), None).unwrap().unwrap();
	let _account_data = AccountData::decode(&mut account_data_opaque.as_slice()).unwrap();
	let _value_proof =
		api.get_storage_value_proof("Balances", "TotalIssuance", None).unwrap().unwrap();
	let _map_proof = api.get_storage_map_proof("System", "Account", &alice, None).unwrap().unwrap();
	let _double_map_proof = api
		.get_storage_double_map_proof(
			"Staking",
			"ErasStakers",
			EraIndex::default(),
			alice_stash,
			None,
		)
		.unwrap()
		.unwrap();
	let _storage_proof =
		api.get_storage_proof_by_keys(vec![storage_key.clone()], None).unwrap().unwrap();
	let _keys = api.get_keys(storage_key, None).unwrap().unwrap();
	let _constants: Balance = api.get_constant("Balances", "ExistentialDeposit").unwrap();
}
