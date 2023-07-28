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

//! This example shows how to use the compose_extrinsic macro to create an extrinsic for any (custom)
//! module, whereas the desired module and call are supplied as a string.

use kitchensink_runtime::AccountId;
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	ac_compose_macros::{compose_call, compose_extrinsic},
	ac_primitives::{
		AssetRuntimeConfig, ExtrinsicSigner as GenericExtrinsicSigner, SignExtrinsic,
		UncheckedExtrinsicV4,
	},
	rpc::WsRpcClient,
	Api, SubmitAndWatch, XtStatus,
};

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let sudoer = AccountKeyring::Alice.pair();
	let client = WsRpcClient::with_default_url();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).unwrap();
	api.set_signer(GenericExtrinsicSigner::<_>::new(sudoer));

	// Compose a call that should only be executable via Sudo.
	let code: Vec<u8> = include_bytes!("integritee_node_runtime-v.compact.wasm").to_vec();
	let call = compose_call!(api.metadata(), "System", "set_code", code);

	let xt: UncheckedExtrinsicV4<_, _, _, _> = compose_extrinsic!(&api, "Sudo", "sudo", call);

	// Send and watch extrinsic until in block.
	let report = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).unwrap();
	println!("[+] Extrinsic got successfully executed {:?}", report.extrinsic_hash);
	println!("[+] Extrinsic got included. Block Hash: {:?}", report.block_hash);
}
