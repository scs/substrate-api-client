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

///! Very simple example that shows how to pretty print the metadata. Has proven to be a helpful
///! debugging tool.

#[macro_use]
extern crate clap;

use clap::App;

use sp_core::sr25519;

use substrate_api_client::utils::hexstr_to_hash;
use substrate_api_client::Api;

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    let api = Api::<sr25519::Pair>::new(format!("ws://{}", url));

    let head = api
        .get_finalized_head()
        .map(|h_str| hexstr_to_hash(h_str).unwrap())
        .unwrap();

    println!("Finalized Head:\n {} \n", head);

    println!(
        "Finalized header:\n {} \n",
        api.get_header(Some(head.clone())).unwrap()
    );

    println!(
        "Finalized block:\n {} \n",
        api.get_block(Some(head)).unwrap()
    );

    println!("Latest Header: \n {} \n", api.get_header(None).unwrap());

    println!("Latest block: \n {} \n", api.get_block(None).unwrap());
}

pub fn get_node_url_from_cli() -> String {
    let yml = load_yaml!("../../src/examples/cli.yml");
    let matches = App::from_yaml(yml).get_matches();

    let node_ip = matches.value_of("node-server").unwrap_or("127.0.0.1");
    let node_port = matches.value_of("node-port").unwrap_or("9944");
    let url = format!("{}:{}", node_ip, node_port);
    println!("Interacting with node on {}\n", url);
    url
}
