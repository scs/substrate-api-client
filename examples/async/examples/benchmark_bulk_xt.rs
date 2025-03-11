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

use resonance_runtime::{BalancesCall, RuntimeCall};
use sp_keyring::Sr25519Keyring;
use substrate_api_client::{
	ac_primitives::{
		Config, ExtrinsicSigner as GenericExtrinsicSigner, RococoRuntimeConfig, SignExtrinsic,
	},
	rpc::JsonrpseeClient,
	Api, SubmitExtrinsic,
};

// Define an extrinsic signer type which sets the generic types of the `GenericExtrinsicSigner`.
// This way, the types don't have to be reassigned with every usage of this type and makes
// the code better readable.
type ExtrinsicSigner = GenericExtrinsicSigner<RococoRuntimeConfig>;

// To access the ExtrinsicAddress type of the Signer, we need to do this via the trait `SignExtrinsic`.
// For better code readability, we define a simple type here and, at the same time, assign the
// AccountId type of the `SignExtrinsic` trait.
type ExtrinsicAddressOf<Signer> = <Signer as SignExtrinsic<AccountId>>::ExtrinsicAddress;

// AccountId type of rococo runtime.
type AccountId = <RococoRuntimeConfig as Config>::AccountId;

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let signer = Sr25519Keyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<RococoRuntimeConfig, _>::new(client).await.unwrap();
	api.set_signer(signer.into());

	let recipient: ExtrinsicAddressOf<ExtrinsicSigner> = Sr25519Keyring::Bob.to_account_id().into();
	// We use a manual nonce input here, because otherwise the api retrieves the nonce via getter and needs
	// to wait for the response of the node (and the actual execution of the previous extrinsic).
	// But because we want to spam the node with extrinsic, we simple monotonically increase the nonce, without
	// waiting for the response of the node.
	let mut nonce = api.get_nonce().await.unwrap();
	let first_nonce = nonce;
	while nonce < first_nonce + 500 {
		// Compose a balance extrinsic.
		let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
			dest: recipient.clone(),
			value: 1_000_000,
		});
		let xt = api.compose_extrinsic_offline(call, nonce);

		println!("Sending extrinsic with nonce {}", nonce);
		let _tx_hash = api.submit_extrinsic(xt).await.unwrap();

		nonce += 1;
	}
}
