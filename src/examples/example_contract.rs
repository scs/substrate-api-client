#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate log;

use std::sync::mpsc::{channel, Receiver};

use clap::App;
use codec::Decode;
use log::*;
use node_primitives::Hash;
use test_node_runtime::Event;

use substrate_api_client::{
    Api,
    crypto::*,
    extrinsic::{
        contract,
        xt_primitives::GenericAddress,
    },
    utils::*,
};

fn main() {
    env_logger::init();

    let yml = load_yaml!("../../src/examples/cli.yml");
    let matches = App::from_yaml(yml).get_matches();

    let node_ip = matches.value_of("node-server").unwrap_or("127.0.0.1");
    let node_port = matches.value_of("node-port").unwrap_or("9944");
    let url = format!("{}:{}", node_ip, node_port);
    println!("Interacting with node on {}", url);

    let from = AccountKey::new("//Alice", Some(""), CryptoKind::Sr25519);
    let api = Api::new(format!("ws://{}", url))
        .set_signer(from);
    println!("[+] Alice's Account Nonce is {}", api.get_nonce());


    const CONTRACT: &str = r#"
(module
    (func (export "call"))
    (func (export "deploy"))
)
"#;
    let wasm = wabt::wat2wasm(CONTRACT).expect("invalid wabt");
    let xt = contract::put_code(
        api.clone(),
        500_000,
        wasm,
    );

    let mut _xthex = xt.hex_encode();
    println!("[+] Sending Extrinsic. Hash: {}", _xthex);
    let tx_hash = api.send_extrinsic(_xthex).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);

    let (events_in, events_out) = channel();
    api.subscribe_events(events_in.clone());

    let code_hash = subcribe_to_code_stored_event(&events_out);
    println!("[+] Got code hash: {:?}\n", code_hash);

    let xt = contract::create(
        api.clone(),
        500_000,
        500_000,
        code_hash,
        vec![1u8],
    );

    let _xthex = xt.hex_encode();
    println!("[+] Sending Extrinsic. Hash: {}", _xthex);
    let tx_hash = api.send_extrinsic(_xthex).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);

    // Now if the contract has been instantiated successfully, the following events are fired:
    // - indices.NewAccountIndex, balances.NewAccount -> generic events when an account is created
    // - contract.Transfer(from, to, balance) -> Transfer from caller of contract.create/call to the contract account
    // - contract.Instantiated(from, deployedAt) -> successful deployment at address. We Want this one.
    let deployed_at = subscribe_to_code_instantiated_event(&events_out);
    println!("[+] Contract deployed at: {:?}\n", deployed_at);

    let xt = contract::call(
        api.clone(),
        deployed_at,
        500_000,
        500_000,
        vec![1u8],
    );

    let _xthex = xt.hex_encode();
    println!("[+] Sending Extrinsic. Hash: {}", _xthex);
    let tx_hash = api.send_extrinsic(_xthex).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);
}

fn subcribe_to_code_stored_event(events_out: &Receiver<String>) -> Hash {
    loop {
        let event_str = events_out.recv().unwrap();

        let _unhex = hexstr_to_vec(event_str);
        let mut _er_enc = _unhex.as_slice();
        let _events = Vec::<system::EventRecord::<Event, Hash>>::decode(&mut _er_enc);
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

        let _unhex = hexstr_to_vec(event_str);
        let mut _er_enc = _unhex.as_slice();
        let _events = Vec::<system::EventRecord::<Event, Hash>>::decode(&mut _er_enc);
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

