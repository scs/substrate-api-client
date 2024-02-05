/*
	Copyright 2023 Supercomputing Systems AG
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

//! Example that shows how to detect a runtime update and afterwards update the metadata.

use core::{
	sync::atomic::{AtomicBool, Ordering},
	time::Duration,
};
use sp_keyring::AccountKeyring;
use sp_weights::Weight;
use std::{sync::Arc, thread};
use substrate_api_client::{
	ac_compose_macros::{compose_call, compose_extrinsic},
	ac_primitives::{AssetRuntimeConfig, Config},
	api_client::UpdateRuntime,
	rpc::TungsteniteRpcClient,
	rpc_api::RuntimeUpdateDetector,
	Api, SubmitAndWatch, SubscribeEvents, XtStatus,
};

type Hash = <AssetRuntimeConfig as Config>::Hash;

fn main() {
	env_logger::init();

	// Initialize the api.
	let client = TungsteniteRpcClient::with_default_url(1);
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).unwrap();
	let sudoer = AccountKeyring::Alice.pair();
	api.set_signer(sudoer.into());

	let subscription = api.subscribe_events().unwrap();
	let cancellation = Arc::new(AtomicBool::new(false));
	let mut update_detector: RuntimeUpdateDetector<Hash, TungsteniteRpcClient> =
		RuntimeUpdateDetector::new_with_cancellation(subscription, cancellation.clone());

	println!("Current spec_version: {}", api.spec_version());

	let handler = thread::spawn(move || {
		// Wait for potential runtime update events
		let runtime_update_detected = update_detector.detect_runtime_update().unwrap();
		println!("Detected runtime update: {runtime_update_detected}");
		assert!(runtime_update_detected);
	});

	// Execute an actual runtime update
	{
		send_code_update_extrinsic(&api);
	}

	// Sleep for some time in order to wait for a runtime update
	// If no update happens we cancel the wait
	{
		thread::sleep(Duration::from_secs(1));
		cancellation.store(true, Ordering::SeqCst);
	}

	handler.join().unwrap();
	api.update_runtime().unwrap();
	println!("New spec_version: {}", api.spec_version());
	assert!(api.spec_version() == 1268);
}

pub fn send_code_update_extrinsic(
	api: &substrate_api_client::Api<AssetRuntimeConfig, TungsteniteRpcClient>,
) {
	let new_wasm: &[u8] = include_bytes!("kitchensink_runtime.compact.compressed.wasm");

	// Create a sudo `set_code` call.
	let call = compose_call!(api.metadata(), "System", "set_code", new_wasm.to_vec());
	let weight: Weight = 0.into();
	let xt = compose_extrinsic!(&api, "Sudo", "sudo_unchecked_weight", call, weight);

	println!("Sending extrinsic to trigger runtime update");
	let block_hash = api
		.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock)
		.unwrap()
		.block_hash
		.unwrap();
	println!("[+] Extrinsic got included. Block Hash: {:?}", block_hash);
}
