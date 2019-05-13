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

#[macro_use]
extern crate log;
extern crate env_logger;

use substrate_api_client::{Api, hexstr_to_u256};

use keyring::AccountKeyring;
use node_primitives::AccountId;
use parity_codec::Encode;


fn main() {
    env_logger::init();

    let mut api = Api::new("ws://127.0.0.1:9944".to_string());
    api.init();

    // get some plain storage value
    let result_str = api.get_storage("Balances", "TransactionBaseFee", None).unwrap();
    let result = hexstr_to_u256(result_str);
    info!("[+] TransactionBaseFee is {}", result);

    // get Alice's AccountNonce
    let accountid = AccountId::from(AccountKeyring::Alice);
    let result_str = api.get_storage("System", "AccountNonce", Some(accountid.encode())).unwrap();
    let result = hexstr_to_u256(result_str);
    info!("[+] Alice's Account Nonce is {}", result);
}