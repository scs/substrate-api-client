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

use std::convert::TryFrom;

use clap::App;

use sp_core::sr25519;

use substrate_api_client::{Api, Metadata};

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    let api = Api::<sr25519::Pair>::new(url).unwrap();

    let meta = Metadata::try_from(api.get_metadata().unwrap()).unwrap();

    meta.print_overview();
    meta.print_modules_with_calls();
    meta.print_modules_with_events();

    // print full substrate metadata json formatted
    println!(
        "{}",
        Metadata::pretty_format(&api.get_metadata().unwrap())
            .unwrap_or_else(|| "pretty format failed".to_string())
    )
}

pub fn get_node_url_from_cli() -> String {
    let yml = load_yaml!("../../src/examples/cli.yml");
    let matches = App::from_yaml(yml).get_matches();

    let node_ip = matches.value_of("node-server").unwrap_or("ws://127.0.0.1");
    let node_port = matches.value_of("node-port").unwrap_or("9944");
    let url = format!("{}:{}", node_ip, node_port);
    println!("Interacting with node on {}\n", url);
    url
}
