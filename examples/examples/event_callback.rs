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

//! Very simple example that shows how to subscribe to events.

use codec::Decode;
use frame_support::dispatch::DispatchInfo;
use kitchensink_runtime::Runtime;
use log::debug;
use sp_core::H256 as Hash;
use substrate_api_client::{
	rpc::{HandleSubscription, JsonrpseeClient},
	Api, PlainTipExtrinsicParams, SubscribeFrameSystem,
};

// This module depends on node_runtime.
// To avoid dependency collisions, node_runtime has been removed from the substrate-api-client library.
// Replace this crate by your own if you run a custom substrate node to get your custom events.
use kitchensink_runtime::RuntimeEvent;

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api.
	let client = JsonrpseeClient::with_default_url().unwrap();
	let api = Api::<(), _, PlainTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();

	println!("Subscribe to a single event type");
	let (sender, receiver) = channel();
	let api2 = api.clone();
	thread::spawn(move || {
		api2.subscribe_for_event_type::<ExtrinsicSuccess>(sender).unwrap();
	});

	println!("Subscribe to events");
	let mut subscription = api.subscribe_system_events().unwrap();

	// Wait for event callbacks from the node, which are received via subscription.
	for _ in 0..5 {
		let event_bytes = subscription.next().unwrap().unwrap().changes[0].1.clone().unwrap().0;
		let events = Vec::<frame_system::EventRecord<RuntimeEvent, Hash>>::decode(
			&mut event_bytes.as_slice(),
		)
		.unwrap();
		for evr in &events {
			println!("decoded: {:?} {:?}", evr.phase, evr.event);
			match &evr.event {
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
						frame_system::Event::ExtrinsicSuccess { dispatch_info: DispatchInfo } => {
							println!("DispatchInfo: {:?}", dispatch_info);
							return
						},
						_ => {
							debug!("ignoring unsupported system event");
						},
					}
				},
				_ => debug!("ignoring unsupported module event: {:?}", evr.event),
			}
		}
	}

	// After we finished whatever we wanted, unusubscribe from the subscription:
	subscription.unsubscribe();
}
