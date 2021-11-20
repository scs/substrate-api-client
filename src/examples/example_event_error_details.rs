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

use std::sync::mpsc::channel;

use clap::{load_yaml, App};
use codec::Decode;
use keyring::AccountKeyring;
use sp_core::crypto::Pair;
use sp_runtime::app_crypto::sp_core::sr25519;
use sp_runtime::AccountId32 as AccountId;
use sp_runtime::MultiAddress;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, ApiResult, XtStatus};

// Look at the how the transfer event looks like in in the metadata
#[derive(Decode)]
struct TransferEventArgs {
    from: AccountId,
    to: AccountId,
    value: u128,
}

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let from = AccountKeyring::Alice.pair();

    let client = WsRpcClient::new(&url);
    let api = Api::<sr25519::Pair, _>::new(client)
        .map(|api| api.set_signer(from.clone()))
        .unwrap();

    let from_account_id = AccountKeyring::Alice.to_account_id();

    let amount = match api.get_account_data(&from_account_id).unwrap() {
        Some(alice) => {
            println!("[+] Alice's Free Balance is is {}\n", alice.free);
            alice.free
        }
        None => {
            println!("[+] Alice's Free Balance is is 0\n");
            10000000000000000000
        }
    };

    let to = AccountKeyring::Bob.to_account_id();

    match api.get_account_data(&to).unwrap() {
        Some(bob) => println!("[+] Bob's Free Balance is is {}\n", bob.free),
        None => println!("[+] Bob's Free Balance is is 0\n"),
    }
    // generate extrinsic
    let xt = api.balance_transfer(MultiAddress::Id(to.clone()), amount);

    println!(
        "Sending an extrinsic from Alice (Key = {}),\n\nto Bob (Key = {})\n",
        from.public(),
        to
    );
    println!("[+] Composed extrinsic: {:?}\n", xt);

    // send and watch extrinsic until finalized
    let tx_hash = api
        .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
        .unwrap();
    println!("[+] Transaction got included. Hash: {:?}\n", tx_hash);

    //Transfer will failed as Alice want to transfer all her balance. She has not enough money to pay the fee
    let (events_in, events_out) = channel();
    api.subscribe_events(events_in).unwrap();
    let args: ApiResult<TransferEventArgs> =
        api.wait_for_event("Balances", "Transfer", None, &events_out);
    match args {
        Ok(transfer_event) => {
            println!("Transfer event received!!!\n");
            println!("Transactor: {:?}", transfer_event.from);
            println!("Destination: {:?}", transfer_event.to);
            println!("Value: {:?}", transfer_event.value);
        }
        Err(e) => {
            println!(
                "[+] Alice couldn't transfer {} to Bob because {:?}\n",
                amount, e
            )
        }
    }

    // verify that Bob's free Balance haven't changed
    let bob = api.get_account_data(&to).unwrap().unwrap();
    println!("[+] Bob's Free Balance is now {}\n", bob.free);

    // verify that Alice's free Balance decreased: paid fees
    let alice = api.get_account_data(&from_account_id).unwrap().unwrap();
    println!("[+] Alice's Free Balance is now {}\n", alice.free);
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
