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

//! Tests for the pallet transaction payment interface functions.

use codec::Encode;
use sp_keyring::AccountKeyring;
use sp_runtime::traits::GetRuntimeBlockType;
use substrate_api_client::{
	ac_primitives::{Config, SubstrateConfig},
	extrinsic::BalancesExtrinsics,
	rpc::JsonrpseeClient,
	Api, GetBlock, GetTransactionPayment,
};

// This example run against a specific  node.
// We use the substrate kitchensink runtime: the config is a substrate config with the kitchensink runtime block type.
// ! Careful: Most runtimes uses plain as tips, they need a polkadot config.
// For better code readability, we define the config type.
type KitchensinkConfig =
	SubstrateConfig<<kitchensink_runtime::Runtime as GetRuntimeBlockType>::RuntimeBlock>;

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().unwrap();
	let alice_pair = AccountKeyring::Alice.pair();
	let mut api = Api::<KitchensinkConfig, _>::new(client).unwrap();
	api.set_signer(<KitchensinkConfig as Config>::ExtrinsicSigner::new(alice_pair));

	let bob = AccountKeyring::Bob.to_account_id();

	let block_hash = api.get_block_hash(None).unwrap().unwrap();
	let encoded_xt = api.balance_transfer_allow_death(bob.into(), 1000000000000).encode();

	// Tests
	let _fee_details = api
		.get_fee_details(encoded_xt.clone().into(), Some(block_hash))
		.unwrap()
		.unwrap();
	let _payment_info = api.get_payment_info(encoded_xt.into(), Some(block_hash)).unwrap().unwrap();
}
