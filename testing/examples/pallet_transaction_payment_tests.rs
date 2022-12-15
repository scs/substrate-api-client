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

use kitchensink_runtime::Runtime;
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	rpc::JsonrpseeClient, Api, AssetTipExtrinsicParams, GenericAddress, GetBlock,
	GetTransactionPayment,
};

#[tokio::main]
async fn main() {
	// Setup
	let client = JsonrpseeClient::with_default_url().unwrap();
	let alice_pair = AccountKeyring::Alice.pair();
	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(alice_pair);

	let bob = AccountKeyring::Bob.to_account_id();

	let block_hash = api.get_block_hash(None).unwrap().unwrap();
	let xthex_prefixed = api.balance_transfer(GenericAddress::Id(bob), 1000000000000).hex_encode();

	// Tests
	let _fee_details = api.get_fee_details(&xthex_prefixed, Some(block_hash)).unwrap().unwrap();
	let _payment_info = api.get_payment_info(&xthex_prefixed, Some(block_hash)).unwrap().unwrap();
}
