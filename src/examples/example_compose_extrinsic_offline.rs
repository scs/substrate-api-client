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

//! This examples shows how to use the compose_extrinsic_offline macro which generates an extrinsic
//! without asking the node for nonce and does not need to know the metadata

use clap::{App, load_yaml};

use node_runtime::{Call, BalancesCall};

// compose_extrinsic_offline is only found if extrinsic is imported as well ?!?
use substrate_api_client::{
    Api,
    compose_extrinsic_offline,
    crypto::{AccountKey, CryptoKind, Sr25519, Crypto},
    extrinsic,
};

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let from = AccountKey::new("//Alice", Some(""), CryptoKind::Sr25519);
    let api = Api::new(format!("ws://{}", url))
        .set_signer(from);

    println!("[+] Alice's Account Nonce is {}\n", api.get_nonce().unwrap());
    // Fixme: It is ugly that we cannot use Account key as receiver in the current implementation of the
    // Account key as the public key is generic [u8; 32] and substrate has no conversion to sr25519::Public.
    let to = Sr25519::public_from_suri("//Bob", Some(""));

    let xt = compose_extrinsic_offline!(
        api.clone().signer.unwrap(),
	    Call::Balances(BalancesCall::transfer(to.clone().into(), 42)),
	    api.get_nonce().unwrap(),
	    api.genesis_hash,
	    api.runtime_version.spec_version
    );

    println!("[+] Composed Extrinsic:\n {:?}\n", xt);

    //send and watch extrinsic until finalized
    let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
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
