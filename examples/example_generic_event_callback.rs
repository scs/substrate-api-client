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

///! Very simple example that shows how to subscribe to events generically
/// implying no runtime needs to be imported
use std::sync::mpsc::channel;

use clap::{load_yaml, App};
use codec::Decode;
use sp_core::sr25519;
use sp_runtime::AccountId32 as AccountId;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::Api;

// Look at the how the transfer event looks like in in the metadata
#[derive(Decode)]
struct TransferEventArgs {
    from: AccountId,
    to: AccountId,
    value: u128,
}

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    let client = WsRpcClient::new(&url);
    let api = Api::<sr25519::Pair, _>::new(client).unwrap();

    println!("Subscribe to events");
    let (events_in, events_out) = channel();

    api.subscribe_events(events_in).unwrap();
    let args: TransferEventArgs = api
        .wait_for_event("Balances", "Transfer", None, &events_out)
        .unwrap();

    println!("Transactor: {:?}", args.from);
    println!("Destination: {:?}", args.to);
    println!("Value: {:?}", args.value);
}

pub fn get_node_url_from_cli() -> String {
    let yml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yml).get_matches();

    let node_ip = matches.value_of("node-server").unwrap_or("ws://127.0.0.1");
    let node_port = matches.value_of("node-port").unwrap_or("9944");
    let url = format!("{}:{}", node_ip, node_port);
    println!("Interacting with node on {}", url);
    url
}
