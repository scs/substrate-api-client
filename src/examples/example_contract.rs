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

//! This example shows how to use the predefined contract extrinsics found in the extrinsic module.
//! General (slightly outdated) background on how to deploy ink! contract is found here:
//! `https://substrate.dev/docs/en/contracts/deploying-a-contract`
//!
//!
//! *Note*: The runtime module here is not in the generic substrate node. Hence, this example
//! must run against the customized node found in `https://github.com/scs/substrate-test-nodes`.

use std::sync::mpsc::{channel, Receiver};

use clap::{load_yaml, App};
use codec::Decode;
use log::*;
use primitives::H256 as Hash;
// FIXME: this type doesn't include contract events -> example broken (would rely on test-node-runtime which we try 
// to avoid because of a cargo issue https://github.com/rust-lang/cargo/issues/6571)
// If you'd like to use this in your crate, add your node_runtime to dependencies and add
// use my_node_runtime::Event;
use node_runtime::Event;

use substrate_api_client::{
    crypto::*,
    extrinsic::{contract, xt_primitives::GenericAddress},
    utils::*,
    Api,
};

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let from = AccountKey::new("//Alice", Some(""), CryptoKind::Sr25519);
    let api = Api::new(format!("ws://{}", url)).set_signer(from);
    println!("[+] Alice's Account Nonce is {}", api.get_nonce().unwrap());

    // contract to be deployed on the chain
    const CONTRACT: &str = r#"
(module
    (func (export "call"))
    (func (export "deploy"))
)
"#;
    let wasm = wabt::wat2wasm(CONTRACT).expect("invalid wabt");

    // 1. Put the contract code as a wasm blob on the chain
    let xt = contract::put_code(api.clone(), 500_000, wasm);
    println!(
        "[+] Putting contract code on chain with extrinsic:\n\n{:?}\n",
        xt
    );
    let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);

    // setup the events listener for our chain.
    let (events_in, events_out) = channel();
    api.subscribe_events(events_in.clone());

    // wait for the `contract.CodeStored(code_hash)` event, which returns code hash that is needed
    // to define what contract shall be instantiated afterwards.
    println!("[+] Waiting for the contract.Stored event");
    let code_hash = subcribe_to_code_stored_event(&events_out);
    println!("[+] Event was received. Got code hash: {:?}\n", code_hash);

    // 2. Create an actual instance of the contract
    let xt = contract::create(api.clone(), 1_000, 500_000, code_hash, vec![1u8]);

    println!(
        "[+] Creating a contract instance with extrinsic:\n\n{:?}\n",
        xt
    );
    let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);

    // Now if the contract has been instantiated successfully, the following events are fired:
    // - indices.NewAccountIndex, balances.NewAccount -> generic events when an account is created
    // - balances.Transfer(from, to, balance) -> Transfer from caller of contract.create/call to the contract account
    // - contract.Instantiated(from, deployedAt) -> successful deployment at address. We Want this one.
    // Note: Trying to instantiate the same contract with the same data twice does not work. No event is
    // fired correspondingly.
    println!("[+] Waiting for the contract.Instantiated event");
    let deployed_at = subscribe_to_code_instantiated_event(&events_out);
    println!(
        "[+] Event was received. Contract deployed at: {:?}\n",
        deployed_at
    );

    // 3. Call the contract instance
    let xt = contract::call(api.clone(), deployed_at, 500_000, 500_000, vec![1u8]);

    // Currently, a contract call does not fire any events nor interact in any other fashion with
    // the outside world. Only node logs can supply information on the consequences of a contract
    // call. Still, it can be checked if the transaction was successful.
    println!(
        "[+] Calling the contract with extrinsic Extrinsic:\n{:?}\n\n",
        xt
    );
    let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);
}

fn subcribe_to_code_stored_event(events_out: &Receiver<String>) -> Hash {
    loop {
        let event_str = events_out.recv().unwrap();

        let _unhex = hexstr_to_vec(event_str).unwrap();
        let mut _er_enc = _unhex.as_slice();
        let _events = Vec::<system::EventRecord<Event, Hash>>::decode(&mut _er_enc);
        if let Ok(evts) = _events {
            for evr in &evts {
                debug!("decoded: phase {:?} event {:?}", evr.phase, evr.event);
                if let Event::contracts(ce) = &evr.event {
                    if let contracts::RawEvent::CodeStored(code_hash) = &ce {
                        info!("Received Contract.CodeStored event");
                        info!("Codehash: {:?}", code_hash);
                        return code_hash.to_owned();
                    }
                }
            }
        }
    }
}

fn subscribe_to_code_instantiated_event(events_out: &Receiver<String>) -> GenericAddress {
    loop {
        let event_str = events_out.recv().unwrap();

        let _unhex = hexstr_to_vec(event_str).unwrap();
        let mut _er_enc = _unhex.as_slice();
        let _events = Vec::<system::EventRecord<Event, Hash>>::decode(&mut _er_enc);
        if let Ok(evts) = _events {
            for evr in &evts {
                debug!("decoded: phase {:?} event {:?}", evr.phase, evr.event);
                if let Event::contracts(ce) = &evr.event {
                    if let contracts::RawEvent::Instantiated(from, deployed_at) = &ce {
                        info!("Received Contract.Instantiated Event");
                        info!("From: {:?}", from);
                        info!("Deployed at: {:?}", deployed_at);
                        return GenericAddress::from(deployed_at.to_owned().0);
                    }
                }
            }
        }
    }
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
