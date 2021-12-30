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

//! This example is community maintained and not CI tested, therefore it may not work as is.

use std::sync::mpsc::channel;

use clap::{load_yaml, App};
use codec::Decode;
use keyring::AccountKeyring;
use substrate_api_client::{rpc::WsRpcClient, AccountId, Api, XtStatus};

#[derive(Decode)]
struct ContractInstantiatedEventArgs {
    deployer: AccountId,
    contract: AccountId,
}

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let from = AccountKeyring::Alice.pair();
    let client = WsRpcClient::new(&url);
    let api = Api::new(client).map(|api| api.set_signer(from)).unwrap();
    println!("[+] Alice's Account Nonce is {}", api.get_nonce().unwrap());

    // contract to be deployed on the chain
    const CONTRACT: &str = r#"
(module
    (func (export "call"))
    (func (export "deploy"))
)
"#;
    let wasm = wabt::wat2wasm(CONTRACT).expect("invalid wabt");

    let (events_in, events_out) = channel();
    api.subscribe_events(events_in)
        .expect("cannot subscribe to events");

    let xt = api.contract_instantiate_with_code(
        1_000_000_000_000_000,
        500_000,
        wasm,
        vec![1u8],
        vec![1u8],
    );

    println!(
        "[+] Creating a contract instance with extrinsic:\n\n{:?}\n",
        xt
    );
    let tx_hash = api
        .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
        .unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);

    println!("[+] Waiting for the contracts.Instantiated event");

    let args: ContractInstantiatedEventArgs = api
        .wait_for_event("Contracts", "Instantiated", None, &events_out)
        .unwrap();

    println!(
        "[+] Event was received. Contract deployed at: {:?}\n",
        args.contract
    );

    let xt = api.contract_call(args.contract.into(), 500_000, 500_000, vec![0u8]);

    println!(
        "[+] Calling the contract with extrinsic Extrinsic:\n{:?}\n\n",
        xt
    );
    let tx_hash = api
        .send_extrinsic(xt.hex_encode(), XtStatus::Finalized)
        .unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);
}

pub fn get_node_url_from_cli() -> String {
    let yml = load_yaml!("../../src/examples/cli.yml");
    let matches = App::from_yaml(yml).get_matches();

    let node_ip = matches.value_of("node-server").unwrap_or("127.0.0.1");
    let node_port = matches.value_of("node-port").unwrap_or("9944");
    let url = format!("{}:{}", node_ip, node_port);
    println!("Interacting with node on {}", url);
    url
}