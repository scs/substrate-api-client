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
use sp_keyring::AccountKeyring;
use sp_weights::Weight;
use substrate_api_client::{
	ac_compose_macros::{compose_call, compose_extrinsic},
	ac_primitives::{AssetRuntimeConfig, Config},
	api_client::UpdateRuntime,
	rpc::JsonrpseeClient,
	rpc_api::RuntimeUpdateDetector,
	Api, SubmitAndWatch, SubscribeEvents, XtStatus,
};
use tokio::select;
use tokio_util::sync::CancellationToken;

type Hash = <AssetRuntimeConfig as Config>::Hash;

#[cfg(feature = "sync-examples")]
#[tokio::main]
async fn main() {
	println!("This example is for async use-cases. Please see runtime_update_sync.rs for the sync implementation.")
}

#[cfg(not(feature = "sync-examples"))]
pub async fn send_code_update_extrinsic(
	api: &substrate_api_client::Api<AssetRuntimeConfig, JsonrpseeClient>,
) {
	let new_wasm: &[u8] = include_bytes!("kitchensink_runtime.compact.compressed.wasm");

	// this call can only be called by sudo
	let call = compose_call!(api.metadata(), "System", "set_code", new_wasm.to_vec());
	let weight: Weight = 0.into();
	let xt = compose_extrinsic!(&api, "Sudo", "sudo_unchecked_weight", call, weight);

	println!("Sending extrinsic to trigger runtime update");
	let block_hash = api
		.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock)
		.await
		.unwrap()
		.block_hash
		.unwrap();
	println!("[+] Extrinsic got included. Block Hash: {:?}", block_hash);
}

#[cfg(not(feature = "sync-examples"))]
#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api.
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).await.unwrap();
	let sudoer = AccountKeyring::Alice.pair();
	api.set_signer(sudoer.into());

	let subscription = api.subscribe_events().await.unwrap();
	let mut update_detector: RuntimeUpdateDetector<Hash, JsonrpseeClient> =
		RuntimeUpdateDetector::new(subscription);
	println!("Current spec_version: {}", api.spec_version());

	// Create future that informs about runtime update events
	let detector_future = update_detector.detect_runtime_update();

	let token = CancellationToken::new();
	let cloned_token = token.clone();

	// To prevent blocking forever we create another future that cancels the
	// wait after some time
	tokio::spawn(async move {
		tokio::time::sleep(std::time::Duration::from_secs(20)).await;
		cloned_token.cancel();
		println!("Cancelling wait for runtime update");
	});

	send_code_update_extrinsic(&api).await;

	// Wait for one of the futures to resolve and check which one resolved
	let runtime_update_detected = select! {
		_ = token.cancelled() => {
			false
		},
		_ = detector_future => {
			api.update_runtime().await.unwrap();
			true
		},
	};
	println!("Detected runtime update: {runtime_update_detected}");
	println!("New spec_version: {}", api.spec_version());
	assert!(api.spec_version() == 1268);
	assert!(runtime_update_detected);
}
