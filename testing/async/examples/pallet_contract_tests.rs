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

//! This example is community maintained and not CI tested, therefore it may not work as is.

use codec::Decode;
use kitchensink_runtime::AccountId;
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	ac_compose_macros::primitives::AssetRuntimeConfig, ac_node_api::StaticEvent,
	ac_primitives::Determinism, extrinsic::ContractsExtrinsics, rpc::JsonrpseeClient, Api,
	SubmitAndWatch, XtStatus,
};

// To test this example with CI we run it against the Substrate kitchensink node, which uses the asset pallet.
// Therefore, we need to use the `AssetRuntimeConfig` in this example.
// ! However, most Substrate runtimes do not use the asset pallet at all. So if you run an example against your own node
// you most likely should use `DefaultRuntimeConfig` instead.

#[allow(unused)]
#[derive(Decode)]
struct ContractInstantiatedEventArgs {
	deployer: AccountId,
	contract: AccountId,
}

impl StaticEvent for ContractInstantiatedEventArgs {
	const PALLET: &'static str = "Contracts";
	const EVENT: &'static str = "Instantiated";
}

#[tokio::main]
async fn main() {
	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let signer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).await.unwrap();
	api.set_signer(signer.into());

	println!("[+] Alice's Account Nonce is {}", api.get_nonce().await.unwrap());

	let wasm = include_bytes!("flipper.wasm").to_vec();

	let xt = api
		.contract_instantiate_with_code(
			0,
			500_000_000.into(),
			None,
			wasm.clone().into(),
			vec![0].into(),
			vec![0].into(),
		)
		.await
		.unwrap();

	println!("[+] Creating a contract instance \n");
	let result = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).await;
	// Ensure the contract is valid - just doesnt make any changes.
	assert!(format!("{:?}", result).contains("ContractReverted"));

	let xt = api
		.contract_upload_code(wasm.into(), None, Determinism::Enforced)
		.await
		.unwrap();

	let _report = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).await.unwrap();
}
