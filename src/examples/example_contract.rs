#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate log;

use std::sync::mpsc::channel;
use std::thread;

use clap::App;
use codec::{Decode, Encode};
use keyring::AccountKeyring;
use log::*;
use node_primitives::{AccountId, Hash};
use node_runtime::Event;

use substrate_api_client::Api;
use substrate_api_client::extrinsic::contract_put_code;
use substrate_api_client::extrinsic::crypto::*;
use substrate_api_client::utils::*;

fn main() {
    env_logger::init();

    let yml = load_yaml!("../../src/examples/cli.yml");
    let matches = App::from_yaml(yml).get_matches();

    let node_ip = matches.value_of("node-server").unwrap_or("127.0.0.1");
    let node_port = matches.value_of("node-port").unwrap_or("9944");
    let url = format!("{}:{}", node_ip, node_port);
    println!("Interacting with node on {}", url);
    let api = Api::new(format!("ws://{}", url));

    let from = AccountKey::new("//Alice", Some(""), CryptoKind::Sr25519);
    let accountid = AccountId::from(AccountKeyring::Alice);
    let result_str = api.get_storage("System", "AccountNonce", Some(accountid.encode())).unwrap();
    let nonce = hexstr_to_u256(result_str);
    println!("[+] Alice's Account Nonce is {}", nonce);


    const CONTRACT: &str = r#"
(module
    (func (export "call"))
    (func (export "deploy"))
)
"#;
    let wasm = wabt::wat2wasm(CONTRACT).expect("invalid wabt");
    let xt = contract_put_code(from, 500_000, wasm, nonce, api.genesis_hash.clone(), api.metadata.clone());

    let mut _xthex = hex::encode(xt.encode());
    _xthex.insert_str(0, "0x");

    let tx_hash = api.send_extrinsic(_xthex).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);

    let code_hash = subcribe_to_code_stored_event(api.clone());
    println!("[+] Got code hash: {:?}", code_hash);
}

fn subcribe_to_code_stored_event(api: Api) -> Hash {
    let (events_in, events_out) = channel();

    thread::Builder::new()
        .spawn(move || {
            api.subscribe_events(events_in.clone());
        })
        .unwrap();

    loop {
        let event_str = events_out.recv().unwrap();

        let _unhex = hexstr_to_vec(event_str);
        let mut _er_enc = _unhex.as_slice();
        let _events = Vec::<system::EventRecord::< Event, Hash >> ::decode(&mut _er_enc);
        if let Ok(evts) = _events {
            for evr in &evts {
                debug!("decoded: phase {:?} event {:?}", evr.phase, evr.event);
                if let Event::contracts(ce) = &evr.event {
                    if let contracts::RawEvent::CodeStored(code_hash) = &ce {
                        println!("Received CodeStored event");
                        println!("Codehash: {:?}", code_hash);
                        return code_hash.to_owned();
                    }
                }
            }
        }
    }
}

