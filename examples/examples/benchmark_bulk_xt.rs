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

//! This examples floats the node with a series of transactions.

// run this against test node with
// > substrate-test-node --dev --execution native --ws-port 9979 -ltxpool=debug

use codec::Encode;
use kitchensink_runtime::{BalancesCall, Runtime, RuntimeCall};
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	rpc::JsonrpseeClient, Api, AssetTipExtrinsicParams, GenericAddress, SubmitExtrinsic,
};

#[tokio::main]
async fn main() {
	env_logger::init();

	// initialize api and set the signer (sender) that is used to sign the extrinsics
	let from = AccountKeyring::Alice.pair();

	let client = JsonrpseeClient::with_default_url().unwrap();

	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(from);

	// define the recipient
	let to = AccountKeyring::Bob.to_account_id();

	let mut nonce = api.get_nonce().unwrap();
	let first_nonce = nonce;
	while nonce < first_nonce + 500 {
		// compose the extrinsic with all the element
		let call = RuntimeCall::Balances(BalancesCall::transfer {
			dest: GenericAddress::Id(to.clone()),
			value: 1_000_000,
		});
		let xt = api.compose_extrinsic_offline(call, nonce);

		println!("sending extrinsic with nonce {}", nonce);
		let _tx_hash = api.submit_extrinsic(xt.encode()).unwrap();

		nonce += 1;
	}
}
