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
use kitchensink_runtime::Runtime;
use log::debug;
use sp_core::{sr25519, H256 as Hash};
use std::sync::mpsc::channel;

// This module depends on node_runtime.
// To avoid dependency collisions, node_runtime has been removed from the substrate-api-client library.
// Replace this crate by your own if you run a custom substrate node to get your custom events.
use kitchensink_runtime::RuntimeEvent;

#[cfg(feature = "ws-client")]
use substrate_api_client::rpc::WsRpcClient;

#[cfg(feature = "tungstenite-client")]
use substrate_api_client::rpc::TungsteniteRpcClient;

use substrate_api_client::{utils::FromHexString, Api, AssetTipExtrinsicParams};

fn main() {
	env_logger::init();

	#[cfg(feature = "ws-client")]
	let client = WsRpcClient::new("ws://127.0.0.1:9944");

	#[cfg(feature = "tungstenite-client")]
	let client = TungsteniteRpcClient::new(url::Url::parse("ws://127.0.0.1:9944").unwrap(), 100);

	let api =
		Api::<sr25519::Pair, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();

	println!("Subscribe to events");
	let (events_in, events_out) = channel();
	api.subscribe_events(events_in).unwrap();

	for _ in 0..5 {
		let event_str = events_out.recv().unwrap();

		let _unhex = Vec::from_hex(event_str).unwrap();
		let mut _er_enc = _unhex.as_slice();
		let events =
			Vec::<frame_system::EventRecord<RuntimeEvent, Hash>>::decode(&mut _er_enc).unwrap();
		for evr in &events {
			println!("decoded: {:?} {:?}", evr.phase, evr.event);
			match &evr.event {
				RuntimeEvent::Balances(be) => {
					println!(">>>>>>>>>> balances event: {:?}", be);
					match &be {
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
				_ => debug!("ignoring unsupported module event: {:?}", evr.event),
			}
		}
	}
}
