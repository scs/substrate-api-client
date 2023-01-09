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
use kitchensink_runtime::Runtime;
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	rpc::JsonrpseeClient, AccountId, Api, PlainTipExtrinsicParams, StaticEvent, SubmitAndWatch,
	SubscribeEvents, SubscribeFrameSystem, XtStatus,
};

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
	let client = JsonrpseeClient::with_default_url().unwrap();
	// ! Careful: AssetTipExtrinsicParams is used here, because the substrate kitchensink runtime uses assets as tips. But for most
	// runtimes, the PlainTipExtrinsicParams needs to be used.
	let mut api = Api::<_, _, PlainTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(signer);

	println!("[+] Alice's Account Nonce is {}", api.get_nonce().unwrap());

	// contract to be deployed on the chain
	const CONTRACT: &str = r#"
(module
    (func (export "call"))
    (func (export "deploy"))
)
"#;
	let wasm = wabt::wat2wasm(CONTRACT).expect("invalid wabt");

	let mut subscription = api.subscribe_system_events().expect("cannot subscribe to events");

	let xt = api.contract_instantiate_with_code(
		1_000_000_000_000_000,
		500_000,
		wasm,
		vec![1u8],
		vec![1u8],
	);

	println!("[+] Creating a contract instance with extrinsic:\n\n{:?}\n", xt);
	let block_hash = api
		.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock)
		.unwrap()
		.block_hash
		.unwrap();
	println!("[+] Extrinsic is in Block. Hash: {:?}\n", block_hash);

	println!("[+] Waiting for the contracts.Instantiated event");

	let args: ContractInstantiatedEventArgs = api.wait_for_event(&mut subscription).unwrap();

	println!("[+] Event was received. Contract deployed at: {:?}\n", args.contract);

	let xt = api.contract_call(args.contract.into(), 500_000, 500_000, vec![0u8]);

	println!("[+] Calling the contract with extrinsic Extrinsic:\n{:?}\n\n", xt);
	let report = api.submit_and_watch_extrinsic_until(xt, XtStatus::Finalized).unwrap();
	println!("[+] Extrinsic got finalized. Extrinsic Hash: {:?}", report.extrinsic_hash);
}
