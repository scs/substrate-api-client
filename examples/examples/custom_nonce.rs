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
	Api, Error, GetChainInfo, SubmitAndWatch, UnexpectedTxStatus, XtStatus,
};

// To test this example in CI, we run it against the Substrate kitchensink node. Therefore, we use the AssetRuntimeConfig.
// ! Careful: Most runtimes uses plain as tips, they need a polkadot config.

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let signer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).unwrap();
	api.set_signer(ExtrinsicSigner::<AssetRuntimeConfig>::new(signer));

	// Information for Era for mortal transactions.
	let last_finalized_header_hash = api.get_finalized_head().unwrap().unwrap();
	let header = api.get_header(Some(last_finalized_header_hash)).unwrap().unwrap();
	let period = 5;
	let tx_params = GenericAdditionalParams::new()
		.era(Era::mortal(period, header.number.into()), last_finalized_header_hash)
		.tip(0);

	// Set the custom additional params.
	api.set_additional_params(tx_params);

	// Get the nonce of Alice.
	let signer_nonce = api.get_nonce().unwrap();
	println!("[+] Signer's Account Nonce is {}\n", signer_nonce);

	// Create an extrinsic that should get included in the future pool due to a nonce that is too high.
	let recipient = MultiAddress::Id(AccountKeyring::Bob.to_account_id());
	let call =
		RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: recipient, value: 42 });
	let xt = api.compose_extrinsic_offline(call, signer_nonce + 1);
	println!("[+] Composed Extrinsic:\n {:?}\n", xt);

	// Send and watch extrinsic until InBlock.
	let result = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock);
	println!("Returned Result {:?}", result);
	match result {
		Err(Error::UnexpectedTxStatus(UnexpectedTxStatus::Future)) => {
			// All good, we expected a Future Error.
		},
		_ => panic!("Expected a future error"),
	}
}
