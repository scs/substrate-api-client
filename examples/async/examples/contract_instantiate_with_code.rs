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
	extrinsic::ContractsExtrinsics, rpc::JsonrpseeClient, Api, SubmitAndWatch, XtStatus,
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
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let signer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).await.unwrap();
	api.set_signer(signer.into());

	println!("[+] Alice's Account Nonce is {}", api.get_nonce().await.unwrap());

	// contract to be deployed on the chain
	const CONTRACT: &str = r#"
(module
    (func (export "call"))
    (func (export "deploy"))
)
"#;
	let wasm = wabt::wat2wasm(CONTRACT).expect("invalid wabt");

	let xt = api
		.contract_instantiate_with_code(1_000_000_000_000_000, 500_000, wasm, vec![1u8], vec![1u8])
		.await
		.unwrap();

	println!("[+] Creating a contract instance with extrinsic:\n\n{:?}\n", xt);
	let report = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).await.unwrap();
	println!("[+] Extrinsic is in Block. Hash: {:?}\n", report.block_hash.unwrap());

	println!("[+] Waiting for the contracts.Instantiated event");

	let associated_contract_events = report.events.unwrap();

	let contract_instantiated_events: Vec<ContractInstantiatedEventArgs> =
		associated_contract_events
			.iter()
			.filter_map(|event| event.as_event().unwrap())
			.collect();
	// We only expect one instantiated event
	assert_eq!(contract_instantiated_events.len(), 1);
	let contract = contract_instantiated_events[0].contract.clone();
	println!("[+] Event was received. Contract deployed at: {contract:?}\n");

	let xt = api.contract_call(contract.into(), 500_000, 500_000, vec![0u8]).await.unwrap();

	println!("[+] Calling the contract with extrinsic Extrinsic:\n{:?}\n\n", xt);
	let report = api.submit_and_watch_extrinsic_until(xt, XtStatus::Finalized).await.unwrap();
	println!("[+] Extrinsic got finalized. Extrinsic Hash: {:?}", report.extrinsic_hash);
}
