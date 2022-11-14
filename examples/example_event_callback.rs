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

///! Very simple example that shows how to subscribe to events.
use std::sync::mpsc::channel;

use clap::{load_yaml, App};
use codec::Decode;
use log::debug;
use sp_core::{sr25519, H256 as Hash};

// This module depends on node_runtime.
// To avoid dependency collisions, node_runtime has been removed from the substrate-api-client library.
// Replace this crate by your own if you run a custom substrate node to get your custom events.
use kitchensink_runtime::RuntimeEvent;

use substrate_api_client::{rpc::WsRpcClient, utils::FromHexString, Api, AssetTipExtrinsicParams};

fn main() {
	env_logger::init();
	let url = get_node_url_from_cli();

	let client = WsRpcClient::new(&url);
	let api = Api::<sr25519::Pair, _, AssetTipExtrinsicParams>::new(client).unwrap();

	println!("Subscribe to events");
	let (events_in, events_out) = channel();
	api.subscribe_events(events_in).unwrap();

	for _ in 0..5 {
		let event_str = events_out.recv().unwrap();

		let _unhex = Vec::from_hex(event_str).unwrap();
		let mut _er_enc = _unhex.as_slice();
		let events = Vec::<system::EventRecord<RuntimeEvent, Hash>>::decode(&mut _er_enc).unwrap();
		for evr in &events {
			println!("decoded: {:?} {:?}", evr.phase, evr.event);
			match &evr.event {
				RuntimeEvent::Balances(be) => {
					println!(">>>>>>>>>> balances event: {:?}", be);
					match &be {
						balances::Event::Transfer { from, to, amount } => {
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

pub fn get_node_url_from_cli() -> String {
	let yml = load_yaml!("cli.yml");
	let matches = App::from_yaml(yml).get_matches();

	let node_ip = matches.value_of("node-server").unwrap_or("ws://localhost");
	let node_port = matches.value_of("node-port").unwrap_or("9944");
	let url = format!("{}:{}", node_ip, node_port);
	println!("Interacting with node on {}", url);
	url
}
