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

use sp_keyring::AccountKeyring;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::AccountInfo;
use substrate_api_client::Api;

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    let client = WsRpcClient::new(&url);
    let mut api = Api::new(client).unwrap();

    // get some plain storage value
    let result: u128 = api
        .get_storage_value("Balances", "TotalIssuance", None)
        .unwrap()
        .unwrap();
    println!("[+] TotalIssuance is {}", result);

    let proof = api
        .get_storage_value_proof("Balances", "TotalIssuance", None)
        .unwrap();
    println!("[+] StorageValueProof: {:?}", proof);

    // get StorageMap
    let account = AccountKeyring::Alice.public();
    let result: AccountInfo = api
        .get_storage_map("System", "Account", account, None)
        .unwrap()
        .or_else(|| Some(AccountInfo::default()))
        .unwrap();
    println!("[+] AccountInfo for Alice is {:?}", result);

    // get StorageMap key prefix
    let result = api.get_storage_map_key_prefix("System", "Account").unwrap();
    println!("[+] key prefix for System Account map is {:?}", result);

    // get Alice's AccountNonce with api.get_nonce()
    let signer = AccountKeyring::Alice.pair();
    api.signer = Some(signer);
    println!("[+] Alice's Account Nonce is {}", api.get_nonce().unwrap());
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
