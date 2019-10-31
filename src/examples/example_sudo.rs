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

//! This examples shows how to use the compose_extrinsic macro to create an extrinsic for any (custom)
//! module, whereas the desired module and call are supplied as a string.

use clap::{load_yaml, App};
use keyring::AccountKeyring;
use primitives::{sr25519, crypto::Pair};
use codec::{Encode, Compact};
use substrate_api_client::{
    compose_extrinsic, compose_payload, compose_call,
    extrinsic::xt_primitives::{AccountId, UncheckedExtrinsicV3, GenericAddress},
    Api,
};

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let from = AccountKeyring::Alice.pair();
    let api = Api::new(format!("ws://{}", url)).set_signer(from.clone());

    // set the recipient
    let to = AccountId::from(AccountKeyring::Bob);

    let nonce = api.get_nonce().unwrap();
    // this call can only be called by sudo
    let raw_payload = compose_payload!(
        compose_call!(
            api.metadata.clone(),
            "Balances",
            "set_balance",
            GenericAddress::from(to.0.clone()),
            Compact(42 as u128),
            Compact(42 as u128)
        ),
        GenericExtra::new(nonce),
        nonce,
        api.get_genesis_hash(),
        api.get_spec_version()
    );
    let signature = raw_payload.using_encoded(|payload| from.sign(payload));

    let xtsu: UncheckedExtrinsicV3<_, sr25519::Pair>  = compose_extrinsic!(
        api.clone(),
        "Sudo",
        "sudo",
        raw_payload
    );

    // send and watch extrinsic until finalized
    let tx_hash = api.send_extrinsic(xtsu.hex_encode()).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);
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
