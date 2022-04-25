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

///! Example that shows how generate the data to set the new authorities
use clap::{load_yaml, App};
use codec::Encode;
use node_template_runtime::AccountId;
use sp_core::crypto::Ss58Codec;
use sp_core::{crypto::Pair, sr25519};

use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::utils::storage_key;
use substrate_api_client::Api;

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    // Alice's seed: subkey inspect //Alice
    let from: sr25519::Pair = Pair::from_string(
        "0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
        None,
    )
    .unwrap();
    println!("signer account: {}", from.public().to_ss58check());

    let client = WsRpcClient::new(&url);
    let api = Api::new(client)
        .map(|api| api.set_signer(from.clone()))
        .unwrap();

    let mut current_authorities: Vec<AccountId> = api
        .get_storage_value("Aura", "Authorities", None)
        .unwrap()
        .unwrap();

    println!("CurrentAuthorities: {:?}", current_authorities);

    let storage_key = storage_key("Aura", "Authorities");
    let storage_key_hex = format!("0x{}", hex::encode(&storage_key));
    println!("aura.authorities() storage key: {}", storage_key_hex);

    // Ferdie's pubkey: subkey inspect //Ferdie
    let new_authority_id = sr25519::Public::from_string(
        "0xdce091cd8522262089d2e2a11b543cb9c07bed68cbd29b3aededfdcc235d467b",
    )
    .unwrap();

    println!();
    println!("Adding new authority:");
    println!("Public: 0x{:?}", new_authority_id);
    println!("Public (SS58): {:?}", new_authority_id.to_ss58check());

    current_authorities.push(new_authority_id.into());

    // new authority set that can be set with sudo.call(system.setStorage()))
    let new_authority_hex = format!("0x{}", hex::encode(current_authorities.encode()));
    println!("{}: {}", "New authority set", new_authority_hex);
}

pub fn get_node_url_from_cli() -> String {
    let yml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yml).get_matches();

    let node_ip = matches.value_of("node-server").unwrap_or("ws://127.0.0.1");
    let node_port = matches.value_of("node-port").unwrap_or("9944");
    let url = format!("{}:{}", node_ip, node_port);
    println!("Interacting with node on {}\n", url);
    url
}
