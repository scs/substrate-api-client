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

//! This examples floates the node with a series of transactions
// run this against test node with
// > substrate-test-node --dev --execution native --ws-port 9979 -ltxpool=debug

use clap::{load_yaml, App};

use keyring::AccountKeyring;
use test_node_runtime::{BalancesCall, Call};
use primitives::crypto::Pair;

use substrate_api_client::{
    compose_extrinsic_offline, extrinsic::xt_primitives::UncheckedExtrinsicV4, Api, XtStatus
};

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let from = AccountKeyring::Alice.pair();
    let api = Api::new(format!("ws://{}", url)).set_signer(from);

    println!(
        "[+] Alice's Account Nonce is {}\n",
        api.get_nonce().unwrap()
    );

    // define the recipient
    let to = AccountKeyring::Bob.to_account_id();

    let mut nonce = api.get_nonce().unwrap();

    loop {
        // compose the extrinsic with all the element
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
            api.clone().signer.unwrap(),
            Call::Balances(BalancesCall::transfer(to.clone().into(), 1_000_000)),
            nonce,
            api.genesis_hash,
            api.runtime_version.spec_version
        );
        // send and watch extrinsic until finalized
        println!("sending extrinsic with nonce {}", nonce);
        let blockh = api.send_extrinsic(xt.hex_encode(), XtStatus::Ready).unwrap();
        
        nonce += 1;
    }
    
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
