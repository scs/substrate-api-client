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

#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate substrate_api_client;

use clap::App;
use node_primitives::Balance;

// compose_extrinsic is only found if extrinsic is imported as well ?!?
use substrate_api_client::{
    Api,
    compose_extrinsic,
    crypto::{AccountKey, CryptoKind},
    extrinsic,
};

fn main() {
    env_logger::init();

    let yml = load_yaml!("../../src/examples/cli.yml");
    let matches = App::from_yaml(yml).get_matches();

    let node_ip = matches.value_of("node-server").unwrap_or("127.0.0.1");
    let node_port = matches.value_of("node-port").unwrap_or("9944");
    let url = format!("{}:{}", node_ip, node_port);
    println!("Interacting with node on {}", url);

    let from = AccountKey::new("//Alice", Some(""), CryptoKind::Sr25519);
    let api = Api::new(format!("ws://{}", url))
        .set_signer(from);

    // get Alice's AccountNonce
    println!("[+] Alice's Account Nonce is {}", api.get_nonce());

    let to = AccountKey::public_from_suri("//Bob", Some(""), CryptoKind::Sr25519);

    let xt = compose_extrinsic!(
        api.clone(),
        "Balances",
        "transfer",
        GenericAddress::from(to),
        Compact(Balance::from(42 as u128))
    );

    println!("[+] Composed Extrinsic:\n {:?}", xt);

    //send and watch extrinsic until finalized
    let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);
}
