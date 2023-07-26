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

//! Example that shows how to detect a runtime upgrade and afterwards upgrade the metadata.
use kitchensink_runtime::RuntimeEvent;
use sp_core::H256 as Hash;
use substrate_api_client::{
	ac_primitives::{AssetRuntimeConfig, Config},
	api_client::UpdateRuntime,
	rpc::{JsonrpseeClient, Subscribe},
	rpc_api::EventSubscriptionFor,
	Api, SubscribeEvents,
};
use tokio::select;
use tokio_util::sync::CancellationToken;

struct RuntimeUpdateDetector<T, Client>
where
	T: Config,
	Client: Subscribe,
{
	subscription: EventSubscriptionFor<Client, T::Hash>,
}

impl<T, Client> RuntimeUpdateDetector<T, Client>
where
	T: Config,
	Client: Subscribe,
{
	fn new(subscription: EventSubscriptionFor<Client, T::Hash>) -> Self {
		Self { subscription }
	}

	async fn detect_runtime_upgrade(&mut self) {
		'outer: loop {
			let event_records =
				self.subscription.next_events::<RuntimeEvent, Hash>().await.unwrap().unwrap();
			for event_record in &event_records {
				match &event_record.event {
					RuntimeEvent::System(system_event) => match &system_event {
						frame_system::Event::CodeUpdated => {
							println!("********** Detected a runtime upgrade");
							break 'outer
						},
						_ => println!("********** Received a RuntimeEvent"),
					},
					_ => println!("********** Received some unspecified event"),
				}
			}
		}
		//self.subscription.unsubscribe().unwrap();
	}
}

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api.
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).await.unwrap();

	let subscription = api.subscribe_events().await.unwrap();
	let mut upgrade_detector: RuntimeUpdateDetector<AssetRuntimeConfig, JsonrpseeClient> =
		RuntimeUpdateDetector::new(subscription);
	println!("Current spec_version: {}", api.spec_version());
	let detector_future = upgrade_detector.detect_runtime_upgrade();

	let token = CancellationToken::new();
	let cloned_token = token.clone();

	tokio::spawn(async move {
		tokio::time::sleep(std::time::Duration::from_secs(5)).await;
		token.cancel();
		println!("canceling token");
	});

	select! {
		_ = cloned_token.cancelled() => {
			println!("cancelled");
		},
		_ = detector_future => (),
	};

	api.update_runtime().await.unwrap();
	println!("New spec_version: {}", api.spec_version());
}
