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
use sp_keyring::Sr25519Keyring;
use substrate_api_client::{
	ac_primitives::RococoRuntimeConfig, extrinsic::BalancesExtrinsics, rpc::JsonrpseeClient, Api,
	GetChainInfo, GetTransactionPayment,
};

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let alice_pair = Sr25519Keyring::Alice.pair();
	let mut api = Api::<RococoRuntimeConfig, _>::new(client).await.unwrap();
	api.set_signer(alice_pair.into());

	let bob = Sr25519Keyring::Bob.to_account_id();

	let block_hash = api.get_block_hash(None).await.unwrap().unwrap();
	let encoded_xt = api
		.balance_transfer_allow_death(bob.into(), 1000000000000)
		.await
		.unwrap()
		.encode();

	// Tests
	let _fee_details = api
		.get_fee_details(&encoded_xt.clone().into(), Some(block_hash))
		.await
		.unwrap()
		.unwrap();
	let _payment_info = api
		.get_payment_info(&encoded_xt.into(), Some(block_hash))
		.await
		.unwrap()
		.unwrap();
}
