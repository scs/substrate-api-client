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

//! This example shows how to use the compose_extrinsic_offline macro which generates an extrinsic
//! without asking the node for nonce and does not need to know the metadata

use kitchensink_runtime::{BalancesCall, RuntimeCall};
use sp_keyring::AccountKeyring;
use sp_runtime::{generic::Era, MultiAddress};
use substrate_api_client::{
	ac_primitives::{AssetRuntimeConfig, ExtrinsicSigner, GenericAdditionalParams},
	rpc::JsonrpseeClient,
	Api, GetChainInfo, SubmitAndWatch, XtStatus,
};

// To test this example in CI, we run it against the Substrate kitchensink node. Therefore, we use the AssetRuntimeConfig
// ! Careful: Most runtimes uses plain as tips, they need a polkadot config.

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let signer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().unwrap();
	// Api::new(..) is not actually an offline call, but retrieves metadata and other information from the node.
	// If this is not acceptable, use the Api::new_offline(..) function instead. There are no examples for this,
	// because of the constantly changing substrate node. But check out our unit tests - there are Apis created with `new_offline`.
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).unwrap();
	api.set_signer(ExtrinsicSigner::<AssetRuntimeConfig>::new(signer));

	// Information for Era for mortal transactions (online).
	let last_finalized_header_hash = api.get_finalized_head().unwrap().unwrap();
	let header = api.get_header(Some(last_finalized_header_hash)).unwrap().unwrap();
	let period = 5;
	let tx_params = GenericAdditionalParams::new()
		.era(Era::mortal(period, header.number.into()), last_finalized_header_hash)
		.tip(0);

	// Set the additional params.
	api.set_additional_params(tx_params);

	// Get the nonce of the signer account (online).
	let signer_nonce = api.get_nonce().unwrap();
	println!("[+] Alice's Account Nonce is {}\n", signer_nonce);

	// Compose the extrinsic (offline).
	let recipient = MultiAddress::Id(AccountKeyring::Bob.to_account_id());
	let call =
		RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: recipient, value: 42 });
	let xt = api.compose_extrinsic_offline(call, signer_nonce);

	println!("[+] Composed Extrinsic:\n {:?}\n", xt);

	// Send and watch extrinsic until in block (online).
	let block_hash = api
		.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock)
		.unwrap()
		.block_hash
		.unwrap();
	println!("[+] Extrinsic got included in block {:?}", block_hash);
}
