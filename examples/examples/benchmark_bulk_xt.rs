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

use kitchensink_runtime::{AccountId, BalancesCall, RuntimeCall};
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	ac_primitives::{
		DefaultRuntimeConfig, ExtrinsicSigner as GenericExtrinsicSigner, SignExtrinsic,
	},
	rpc::JsonrpseeClient,
	Api, SubmitExtrinsic,
};

// To test this example with CI we run it against the Substrate kitchensink node, which uses the asset pallet.
// Therefore, we need to use the `AssetRuntimeConfig` in this example.
// ! However, most Substrate runtimes do not use the asset pallet at all. So if you run an example against your own node
// you most likely should use `DefaultRuntimeConfig` instead.

// To access the ExtrinsicAddress type of the Signer, we need to do this via the trait `SignExtrinsic`.
// For better code readability, we define a simple type here and, at the same time, assign the
// AccountId type of the `SignExtrinsic` trait.
type ExtrinsicAddressOf<Signer> = <Signer as SignExtrinsic<AccountId>>::ExtrinsicAddress;

#[tokio::main]
async fn main() {
	let alice_signer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<DefaultRuntimeConfig, _>::new(client).unwrap();
	api.set_signer(GenericExtrinsicSigner::<DefaultRuntimeConfig>::new(alice_signer));
}
