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

//! Example that shows how to subscribe to events and do some action
//! upon encounterign them.

use log::debug;
use sp_core::H256 as Hash;
use substrate_api_client::{
	ac_primitives::RococoRuntimeConfig, rpc::JsonrpseeClient, Api, SubscribeEvents,
};

// This module depends on the specific node runtime.
// Replace this crate by your own if you run a custom substrate node to get your custom events.
use resonance_runtime::RuntimeEvent;

// To test this example with CI we run it against the Polkadot Rococo node. Remember to switch the Config to match your
// own runtime if it uses different parameter configurations. Several pre-compiled runtimes are available in the ac-primitives crate.

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api.
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let api = Api::<RococoRuntimeConfig, _>::new(client).await.unwrap();

	println!("Subscribe to events");
	let mut subscription = api.subscribe_events().await.unwrap();

	// Wait for event callbacks from the node, which are received via subscription.
	for _ in 0..5 {
		let event_records =
			subscription.next_events::<RuntimeEvent, Hash>().await.unwrap().unwrap();
		for event_record in &event_records {
			println!("decoded: {:?} {:?}", event_record.phase, event_record.event);
			match &event_record.event {
				RuntimeEvent::Balances(balances_event) => {
					println!(">>>>>>>>>> balances event: {:?}", balances_event);
					match &balances_event {
						pallet_balances::Event::Transfer { from, to, amount } => {
							println!("Transactor: {:?}", from);
							println!("Destination: {:?}", to);
							println!("Value: {:?}", amount);
							return
						},
						_ => {
							debug!("ignoring unsupported balances event");
						},
					}
				},
				RuntimeEvent::System(system_event) => {
					println!(">>>>>>>>>> system event: {:?}", system_event);
					match &system_event {
						frame_system::Event::ExtrinsicSuccess { dispatch_info } => {
							println!("DispatchInfo: {:?}", dispatch_info);
							return
						},
						_ => {
							debug!("ignoring unsupported system event");
						},
					}
				},
				_ => debug!("ignoring unsupported module event: {:?}", event_record.event),
			}
		}
	}

	// After we finished whatever we wanted, unusubscribe from the subscription,
	// to ensure, that the node does not keep sending us events.
	subscription.unsubscribe().await.unwrap();
}
