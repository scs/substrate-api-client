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

extern crate substrate_api_client;

use substrate_api_client::{Api, hexstr_to_u256, extrinsic::transfer};

use keyring::AccountKeyring;
use node_primitives::AccountId;
use parity_codec::Encode;
use primitive_types::U256;

fn main() {
    let mut api = Api::new("ws://127.0.0.1:9944".to_string());
    api.init();

    // get Alice's AccountNonce
    let accountid = AccountId::from(AccountKeyring::Alice);
    let result_str = api.get_storage("System", "AccountNonce", Some(accountid.encode())).unwrap();
    let nonce = hexstr_to_u256(result_str);
    println!("[+] Alice's Account Nonce is {}", nonce);

    // generate extrinsic
    let xt= transfer("//Alice", "//Bob", U256::from(42), nonce, api.genesis_hash.unwrap());
    println!("extrinsic: {:?}", xt);

    //send and watch extrinsic until finalized
    let tx_hash = api.send_extrinsic(xt).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);
}