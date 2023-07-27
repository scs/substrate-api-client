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
use substrate_api_client::{
	ac_primitives::AssetRuntimeConfig, api_client::UpdateRuntime, rpc::JsonrpseeClient,
	rpc_api::RuntimeUpdateDetector, Api, SubscribeEvents,
};
use tokio::select;
use tokio_util::sync::CancellationToken;

#[cfg(not(feature = "sync-examples"))]
#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api.
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).await.unwrap();

	let subscription = api.subscribe_events().await.unwrap();
	let mut update_detector: RuntimeUpdateDetector<AssetRuntimeConfig, JsonrpseeClient> =
		RuntimeUpdateDetector::new(subscription);
	println!("Current spec_version: {}", api.spec_version());
	let detector_future = update_detector.detect_runtime_update();

	let token = CancellationToken::new();
	let cloned_token = token.clone();

	tokio::spawn(async move {
		tokio::time::sleep(std::time::Duration::from_secs(5)).await;
		cloned_token.cancel();
		println!("Canceling wait for runtime update");
	});

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
}

#[cfg(feature = "sync-examples")]
#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api.
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).unwrap();

	let subscription = api.subscribe_events().unwrap();
	let mut update_detector: RuntimeUpdateDetector<AssetRuntimeConfig, JsonrpseeClient> =
		RuntimeUpdateDetector::new(subscription);
	println!("Current spec_version: {}", api.spec_version());
	let runtime_update_detected = update_detector.detect_runtime_update();

	api.update_runtime().unwrap();

	println!("Detected runtime update: {runtime_update_detected}");
	println!("New spec_version: {}", api.spec_version());
}
