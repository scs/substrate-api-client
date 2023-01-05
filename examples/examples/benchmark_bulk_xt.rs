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

//! This example floods the node with a series of transactions.

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

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let signer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().unwrap();
	// ! Careful: AssetTipExtrinsicParams is used here, because the substrate kitchensink runtime uses assets as tips. But for most
	// runtimes, the PlainTipExtrinsicParams needs to be used.
	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(signer);

	let recipient = AccountKeyring::Bob.to_account_id();
	// We use a manual nonce input here, because otherwise the api retrieves the nonce via getter and needs
	// to wait for the response of the node (and the actual execution of the previous extrinsic).
	// But because we want to spam the node with extrinsic, we simple monotonically increase the nonce, without
	// waiting for the response of the node.
	let mut nonce = api.get_nonce().unwrap();
	let first_nonce = nonce;
	while nonce < first_nonce + 500 {
		// Compose a balance extrinsic.
		let call = RuntimeCall::Balances(BalancesCall::transfer {
			dest: GenericAddress::Id(recipient.clone()),
			value: 1_000_000,
		});
		let xt = api.compose_extrinsic_offline(call, nonce);

		println!("Sending extrinsic with nonce {}", nonce);
		let _tx_hash = api.submit_extrinsic(xt.encode()).unwrap();

		nonce += 1;
	}
}
