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
use std::{sync::Arc, thread};
use substrate_api_client::{
	ac_primitives::AssetRuntimeConfig, api_client::UpdateRuntime, rpc::JsonrpseeClient,
	rpc_api::RuntimeUpdateDetector, Api, SubscribeEvents,
};

#[cfg(not(feature = "sync-examples"))]
#[tokio::main]
async fn main() {
	println!("This example is for sync use-cases. Please see runtime_update_async.rs for the async implementation.")
}

#[cfg(feature = "sync-examples")]
#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api.
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).unwrap();

	let subscription = api.subscribe_events().unwrap();
	let cancellation = Arc::new(AtomicBool::new(false));
	let mut update_detector: RuntimeUpdateDetector<AssetRuntimeConfig, JsonrpseeClient> =
		RuntimeUpdateDetector::new_with_cancellation(subscription, cancellation.clone());

	println!("Current spec_version: {}", api.spec_version());

	let handler = thread::spawn(move || {
		let runtime_update_detected = update_detector.detect_runtime_update();
		println!("Detected runtime update: {runtime_update_detected}");
	});

	thread::sleep(Duration::from_secs(5));
	cancellation.store(true, Ordering::SeqCst);
	handler.join().unwrap();
	api.update_runtime().unwrap();
	println!("New spec_version: {}", api.spec_version());
}
