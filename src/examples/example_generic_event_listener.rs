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
use primitives::sr25519;
use node_primitives::AccountId;

use substrate_api_client::Api;

type TransferArgs = (AccountId, AccountId, u128, u128);

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    let api = Api::<sr25519::Pair>::new(format!("ws://{}", url));

    println!("Subscribe to events");
    let (events_in, events_out) = channel();

    api.subscribe_events(events_in.clone());
    let args: TransferArgs = api.wait_for_event(
        "Balances",
        "Transfer",
        events_out)
        .unwrap()
        .unwrap();

    println!("Transactor: {:?}", args.0);
    println!("Destination: {:?}", args.1);
    println!("Value: {:?}", args.2);
    println!("Fee: {:?}", args.3);
}

pub fn get_node_url_from_cli() -> String {
    let yml = load_yaml!("../../src/examples/cli.yml");
    let matches = App::from_yaml(yml).get_matches();

    let node_ip = matches.value_of("node-server").unwrap_or("127.0.0.1");
    let node_port = matches.value_of("node-port").unwrap_or("9944");
    let url = format!("{}:{}", node_ip, node_port);
    println!("Interacting with node on {}", url);
    url
}
