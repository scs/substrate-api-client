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

///! Example that shows how to compose a setCode extrinsic.
/// Upgrade from node 1.05 to 1.0.6: use wasm integritee_node_runtime-v6.compact.wasm
use clap::{load_yaml, App};
use sp_core::crypto::Ss58Codec;
use sp_core::{crypto::Pair, sr25519};
use support::weights::Weight;

use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{compose_call, compose_extrinsic, Api, UncheckedExtrinsicV4, XtStatus};

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

    //let new_wasm: &[u8] = include_bytes!("integritee_node_runtime-v6.compact.wasm");
    let new_wasm: &[u8] = include_bytes!("integritee_runtime-v12.compact.compressed.wasm");

    // this call can only be called by sudo
    #[allow(clippy::redundant_clone)]
    let call = compose_call!(
        api.metadata.clone(),
        "System",
        "set_code",
        new_wasm.to_vec()
    );

    let weight: Weight = 0;

    #[allow(clippy::redundant_clone)]
    let xt: UncheckedExtrinsicV4<_> =
        compose_extrinsic!(api.clone(), "Sudo", "sudo_unchecked_weight", call, weight);

    // send and watch extrinsic until finalized
    let tx_hash = api
        .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
        .unwrap();

    println!("[+] Transaction got included. Hash: {:?}", tx_hash);
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
