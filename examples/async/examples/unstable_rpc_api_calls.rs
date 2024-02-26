/*
	Copyright 2024 Supercomputing Systems AG
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

//! This example shows how to use the compose_extrinsic_offline macro which generates an extrinsic
//! without asking the node for nonce and does not need to know the metadata

use kitchensink_runtime::{BalancesCall, RuntimeCall};
use sp_keyring::AccountKeyring;
use sp_runtime::{generic::Era, MultiAddress};
use substrate_api_client::{
	ac_primitives::{AssetRuntimeConfig, GenericAdditionalParams},
	rpc::JsonrpseeClient,
	Api, Error, GetChainInfo, SubmitAndWatch, UnexpectedTxStatus, XtStatus,
};

// To test this example with CI we run it against the Substrate kitchensink node, which uses the asset pallet.
// Therefore, we need to use the `AssetRuntimeConfig` in this example.
// ! However, most Substrate runtimes do not use the asset pallet at all. So if you run an example against your own node
// you most likely should use `DefaultRuntimeConfig` instead.

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let signer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).await.unwrap();
	api.set_signer(signer.into());

    let chain_name: String =
			api.client().request("chainSpec_v1_chainName", rpc_params![]).await.unwrap();

    println!("{chain_name}");

    let pending_transactions: Vec<String> =
			api.client().request("sudo_unstable_pendingTransactions", rpc_params![]).await.unwrap();

    println!("{pending_transactions}");

    let version: String =
			api.client().request("sudo_unstable_version", rpc_params![]).await.unwrap();

    println!("{version}");



}
