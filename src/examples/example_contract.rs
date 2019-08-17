#[macro_use]
extern crate clap;
extern crate env_logger;

use clap::App;

use codec::Encode;
use keyring::AccountKeyring;
use node_primitives::AccountId;

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
}
