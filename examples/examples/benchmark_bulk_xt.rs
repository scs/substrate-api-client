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

use kitchensink_runtime::{AccountId, BalancesCall, RuntimeCall};
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	ac_primitives::{AssetRuntimeConfig, ExtrinsicSigner as GenericExtrinsicSigner, SignExtrinsic},
	rpc::JsonrpseeClient,
	Api, SubmitExtrinsic,
};

// To test this example in CI, we run it against the Substrate kitchensink node. Therefore, we use the AssetRuntimeConfig
// ! Careful: Most runtimes uses plain as tips, they need a polkadot config.

// Define an extrinsic signer type which sets the generic types of the `GenericExtrinsicSigner`.
// This way, the types don't have to be reassigned with every usage of this type and makes
// the code better readable.
type ExtrinsicSigner = GenericExtrinsicSigner<AssetRuntimeConfig>;

// To access the ExtrinsicAddress type of the Signer, we need to do this via the trait `SignExtrinsic`.
// For better code readability, we define a simple type here and, at the same time, assign the
// AccountId type of the `SignExtrinsic` trait.
type ExtrinsicAddressOf<Signer> = <Signer as SignExtrinsic<AccountId>>::ExtrinsicAddress;

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let signer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).unwrap();
	api.set_signer(ExtrinsicSigner::new(signer));

	let recipient: ExtrinsicAddressOf<ExtrinsicSigner> = AccountKeyring::Bob.to_account_id().into();
	// We use a manual nonce input here, because otherwise the api retrieves the nonce via getter and needs
	// to wait for the response of the node (and the actual execution of the previous extrinsic).
	// But because we want to spam the node with extrinsic, we simple monotonically increase the nonce, without
	// waiting for the response of the node.
	let mut nonce = api.get_nonce().unwrap();
	let first_nonce = nonce;
	while nonce < first_nonce + 500 {
		// Compose a balance extrinsic.
		let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
			dest: recipient.clone(),
			value: 1_000_000,
		});
		let xt = api.compose_extrinsic_offline(call, nonce);

		println!("Sending extrinsic with nonce {}", nonce);
		let _tx_hash = api.submit_extrinsic(xt).unwrap();

		nonce += 1;
	}
}
