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
#![macro_use]

#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_derive;

use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as ThreadOut;

use codec::{Decode, Encode};
use metadata::RuntimeMetadataPrefixed;
use node_primitives::Hash;
use ws::Result as WsResult;

use crypto::AccountKey;
use extrinsic::xt_primitives::GenericAddress;
use node_metadata::NodeMetadata;
use rpc::json_req;
use utils::*;

#[macro_use]
pub mod extrinsic;
pub mod crypto;
pub mod node_metadata;
pub mod utils;
pub mod rpc;

#[derive(Clone)]
pub struct Api {
    url: String,
    pub signer: Option<AccountKey>,
    pub genesis_hash: Hash,
    pub metadata: NodeMetadata,
}

impl Api {
    pub fn new(url: String) -> Self {
        let genesis_hash = Api::_get_genesis_hash(url.clone());
        info!("Got genesis hash: {:?}", genesis_hash);

        let meta = Api::_get_metadata(url.clone());
        let metadata = node_metadata::parse_metadata_into_module_and_call(&meta);
        info!("Metadata: {:?}", metadata);

        Self { url, signer: None, genesis_hash, metadata }
    }

    pub fn set_signer(mut self, signer: AccountKey) -> Self {
        self.signer = Some(signer);
        self
    }

    fn _get_genesis_hash(url: String) -> Hash {
        let jsonreq = json_req::chain_get_block_hash();
        let genesis_hash_str = Api::_get_request(url.clone() ,jsonreq.to_string()).expect("Fetching genesis hash from node failed");
        hexstr_to_hash(genesis_hash_str)
    }

    fn _get_metadata(url: String) -> RuntimeMetadataPrefixed{
        let jsonreq = json_req::state_get_metadata();
        let metadata_str = Api::_get_request(url,jsonreq.to_string()).unwrap();

        let _unhex = hexstr_to_vec(metadata_str);
        let mut _om = _unhex.as_slice();
        RuntimeMetadataPrefixed::decode(&mut _om).unwrap()
    }

    fn _get_nonce(url: String, signer: AccountKey) -> u32 {
        let result_str = Api::_get_storage(url, "System", "AccountNonce", Some(signer.public().encode())).unwrap();
        let nonce = hexstr_to_u256(result_str);
        nonce.low_u32()
    }

    fn _get_storage(url: String, module: &str, storage_key_name: &str, param: Option<Vec<u8>>) -> WsResult<String> {
        let keyhash = storage_key_hash(module, storage_key_name, param);

        debug!("with storage key: {}", keyhash);
        let jsonreq = json_req::state_get_storage(&keyhash);
        Api::_get_request(url, jsonreq.to_string())
    }

    // low level access
    fn _get_request(url: String, jsonreq: String) -> WsResult<String> {
        let (result_in, result_out) = channel();
        rpc::get(url, jsonreq.clone(), result_in.clone());

        Ok(result_out.recv().unwrap())
    }

    pub fn get_metadata(&self) -> RuntimeMetadataPrefixed {
        Api::_get_metadata(self.url.clone())
    }

    pub fn get_nonce(&self) -> u32 {
        match &self.signer {
            Some(key) =>  Api::_get_nonce(self.url.clone(), key.to_owned()),
            None => panic!("Can't get nonce when no signer is set"),
        }
    }

    pub fn get_request(&self, jsonreq: String) -> WsResult<String> {
        Api::_get_request(self.url.clone(), jsonreq)
    }

    pub fn get_storage(&self, module: &str, storage_key_name: &str, param: Option<Vec<u8>>) -> WsResult<String> {
        Api::_get_storage(self.url.clone(), module, storage_key_name, param)
    }

    pub fn send_extrinsic(&self, xthex_prefixed: String) -> WsResult<Hash> {
        debug!("sending extrinsic: {:?}", xthex_prefixed);

        let jsonreq = json_req::author_submit_and_watch_extrinsic(&xthex_prefixed).to_string();

        let (result_in, result_out) = channel();
        rpc::send_extrinsic_and_wait_until_finalized(self.url.clone(),
                                                          jsonreq.clone(),
                                                          result_in.clone());

        Ok(hexstr_to_hash(result_out.recv().unwrap()))
    }

    pub fn subscribe_events(&self, sender: ThreadOut<String>) {
        debug!("subscribing to events");
        let key = storage_key_hash("System", "Events", None);
        let jsonreq = json_req::state_subscribe_storage(&key).to_string();

        rpc::start_event_subscriber(self.url.clone(),
                                         jsonreq.clone(),
                                         sender.clone());
    }
}
