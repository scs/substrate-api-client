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

///! Very simple example that shows how to get some simple storage values.
use clap::{load_yaml, App};

use keyring::AccountKeyring;
use substrate_api_client::{Api, Hash};

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    let mut api = Api::new(format!("ws://{}", url));

    // get some plain storage value
    let result: u128 = api
        .get_storage_value("Balances", "TotalIssuance", None)
        .unwrap();
    println!("[+] TotalIssuance is {}", result);

    let proof = api
        .get_storage_value_proof("Balances", "TotalIssuance", None)
        .unwrap();
    println!("[+] StorageValueProof: {:?}", proof);

    // get StorageMap
    let result: Hash = api
        .get_storage_map("System", "BlockHash", 1u32, None)
        .or_else(|| Some(Hash::default()))
        .unwrap();
    println!("[+] block hash for blocknumber 42 is {:?}", result);

    // get StorageMap key prefix
    let result = api.get_storage_map_key_prefix("System", "BlockHash");
    println!("[+] key prefix for System BlockHash map is {:?}", result);

    // get StorageDoubleMap
    let result: u32 = api
        .get_storage_double_map("TemplateModule", "SomeDoubleMap", 1_u32, 2_u32, None)
        .or(Some(0))
        .unwrap();
    println!("[+] some double map (1,2) should be 3. Is {:?}", result);

    // get Alice's AccountNonce with api.get_nonce()
    let signer = AccountKeyring::Alice.pair();
    api.signer = Some(signer);
    println!("[+] Alice's Account Nonce is {}", api.get_nonce().unwrap());
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
