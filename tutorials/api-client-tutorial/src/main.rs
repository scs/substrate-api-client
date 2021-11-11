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

use codec::{Decode, Encode};
use sp_core::crypto::Pair;
use sp_keyring::AccountKeyring;

use substrate_api_client::{
    compose_extrinsic, rpc::WsRpcClient, utils::FromHexString, Api, UncheckedExtrinsicV4, XtStatus,
};

#[derive(Encode, Decode, Debug)]
struct Kitty {
    id: [u8; 32],
    price: u128,
}

fn main() {
    let url = "ws://127.0.0.1:9944";

    let signer = AccountKeyring::Alice.pair();
    let client = WsRpcClient::new(url);

    let api = Api::new(client)
        .map(|api| api.set_signer(signer.clone()))
        .unwrap();

    let xt: UncheckedExtrinsicV4<_> =
        compose_extrinsic!(api, "KittyModule", "create_kitty", 10_u128);

    println!("[+] Extrinsic: {:?}\n", xt);

    let tx_hash = api
        .send_extrinsic(xt.hex_encode(), XtStatus::Finalized)
        .unwrap()
        .unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);

    // get the index at which Alice's Kitty resides. Alternatively, we could listen to the StoredKitty
    // event similar to what we do in the example_contract.
    let index: u64 = api
        .get_storage_map("Kitty", "KittyIndex", Some(signer.public().encode()), None)
        .unwrap()
        .unwrap();

    println!("[+] Alice's Kitty is at index : {}\n", index);

    // get the Kitty
    let res_str = api
        .get_storage_map("Kitty", "Kitties", Some(index.encode()), None)
        .unwrap()
        .unwrap();

    let res_vec = Vec::from_hex(res_str).unwrap();

    // type annotations are needed here to know that to decode into.
    let kitty: Kitty = Decode::decode(&mut res_vec.as_slice()).unwrap();
    println!("[+] Cute decoded Kitty: {:?}\n", kitty);
}
