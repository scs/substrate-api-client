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

use clap::{load_yaml, App};

use ac_primitives::PlainTipExtrinsicParamsBuilder;
use node_template_runtime::{BalancesCall, Call, Header};
use sp_core::crypto::Pair;
use sp_keyring::AccountKeyring;
use sp_runtime::generic::Era;
use sp_runtime::MultiAddress;

use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::ExtrinsicParams;
use substrate_api_client::{
    compose_extrinsic_offline, Api, PlainTipExtrinsicParams, UncheckedExtrinsicV4, XtStatus,
};

fn main() {
    env_logger::init();
    let url = get_node_url_from_cli();

    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let from = AccountKeyring::Alice.pair();
    let client = WsRpcClient::new(&url);

    let api = Api::<_, _, PlainTipExtrinsicParams>::new(client)
        .map(|api| api.set_signer(from))
        .unwrap();

    // Information for Era for mortal transactions
    let head = api.get_finalized_head().unwrap().unwrap();
    let h: Header = api.get_header(Some(head)).unwrap().unwrap();
    let period = 5;

    println!(
        "[+] Alice's Account Nonce is {}\n",
        api.get_nonce().unwrap()
    );

    // define the recipient
    let to = MultiAddress::Id(AccountKeyring::Bob.to_account_id());

    let tx_params = PlainTipExtrinsicParamsBuilder::new()
        .era(Era::mortal(period, h.number.into()), api.genesis_hash)
        .tip(0);

    let updated_api = api.set_extrinsic_params(tx_params);

    // compose the extrinsic with all the element
    #[allow(clippy::redundant_clone)]
    let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
        updated_api.clone().signer.unwrap(),
        Call::Balances(BalancesCall::transfer {
            dest: to.clone(),
            value: 42
        }),
        updated_api.get_nonce().unwrap(),
        updated_api.genesis_hash,
        head,
        updated_api.runtime_version.spec_version,
        updated_api.runtime_version.transaction_version,
        updated_api.extrinsic_params
    );

    println!("[+] Composed Extrinsic:\n {:?}\n", xt);

    //println!("[+] Encode Extrinsic:\n {:?}\n", xt.hex_encode());

    // send and watch extrinsic until in block
    let blockh = updated_api
        .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
        .unwrap();
    println!("[+] Transaction got included in block {:?}", blockh);
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
