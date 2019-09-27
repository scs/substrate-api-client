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

///! Very simple example that shows how to use a predefined extrinsic from the extrinsic module
use clap::{load_yaml, App};

use substrate_api_client::{
    crypto::{AccountKey, CryptoKind},
    extrinsic,
    extrinsic::xt_primitives::*,
    Api,
};

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let from = AccountKey::new("//Alice", Some(""), CryptoKind::Sr25519);
    let api = Api::new(format!("ws://{}", url)).set_signer(from.clone());

    let to = AccountKey::public_from_suri("//Bob", Some(""), CryptoKind::Sr25519);

    let result = api.get_free_balance(to);
    println!("[+] Bob's Free Balance is is {}\n", result);

    // generate extrinsic
    let xt = extrinsic::balances::transfer(api.clone(), GenericAddress::from(to), 1000);

    println!(
        "Sending an extrinsic from Alice (Key = {:?}),\n\nto Bob (Key = {:?})\n",
        from.public(),
        to
    );

    println!("[+] Composed extrinsic: {:?}\n", xt);

    // send and watch extrinsic until finalized
    let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);

    // verify that Bob's free Balance increased
    let result = api.get_free_balance(to);
    println!("[+] Bob's Free Balance is now {}\n", result);
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
