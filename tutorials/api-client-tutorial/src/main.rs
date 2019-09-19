use codec::{Decode, Encode};
use substrate_api_client::{
    Api,
    compose_extrinsic,
    crypto::{AccountKey, CryptoKind},
    extrinsic,
    utils::{hexstr_to_u64, hexstr_to_vec}
};

#[derive(Encode, Decode, Debug)]
struct Kitty {
    id: [u8; 32],
    price: u128,
}

fn main() {
    let url = "127.0.0.1:9944";

    let signer = AccountKey::new("//Alice", Some(""), CryptoKind::Sr25519);

    let api = Api::new(format!("ws://{}", url))
        .set_signer(signer.clone());

    let xt = compose_extrinsic!(
        api.clone(),
        "KittyModule",
        "create_kitty",
        10 as u128
    );

    println!("[+] Extrinsic: {:?}\n", xt);

    let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);

    // Get the index at which Alice's Kitty resides. Alternatively, we could listen to the StoredKitty
    // event similar to what we do in the example_contract.
    let res_str = api.get_storage("Kitty",
                                  "KittyIndex",
                                  Some(signer.public().encode())).unwrap();

    let index = hexstr_to_u64(res_str).unwrap();
    println!("[+] Alice's Kitty is at index : {}\n", index);

    // Get the Kitty
    let res_str = api.get_storage("Kitty",
                                  "Kitties",
                                  Some(index.encode())).unwrap();

    let res_vec = hexstr_to_vec(res_str).unwrap();

    // Type annotations are needed here to know that to decode into.
    let kitty: Kitty = Decode::decode(&mut res_vec.as_slice()).unwrap();
    println!("[+] Cute decoded Kitty: {:?}\n", kitty);
}
