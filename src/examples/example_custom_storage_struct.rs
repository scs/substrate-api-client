#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate log;

use clap::App;
use codec::{Decode, Encode};
use log::*;
use primitives::H256;

use substrate_api_client::{
    Api,
    compose_extrinsic,
    crypto::{AccountKey, CryptoKind},
    extrinsic,
    utils::*,
};

#[derive(Encode, Decode, Debug)]
struct Kitty {
    id: H256,
    price: u128,
}

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
        .set_signer(from.clone());

    let xt = compose_extrinsic!(
        api.clone(),
        "KittyModule",
        "create_kitty",
        10 as u128
    );

    println!("[+] Composed Extrinsic:\n {:?}", xt);
    //send and watch extrinsic until finalized
    let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);

    let res_str = api.get_storage("Kitty",
                                  "KittyIndex",
                                  Some(from.public().encode())).unwrap();
    let index = hexstr_to_u64(res_str);
    println!("[+] Created Kitty is at: {}", index);


    let res_str = api.get_storage("Kitty",
                                  "Kitties",
                                  Some(index.encode())).unwrap();
    println!("[+] Got kitty result str: {}", res_str);
    let res_slice = hexstr_to_vec(res_str);
    let kitty: Kitty = Decode::decode(&mut res_slice.as_slice()).unwrap();

    println!("Decoded Kitty: {:?}", kitty);
}

