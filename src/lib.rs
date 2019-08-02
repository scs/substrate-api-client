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

#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as ThreadOut;

use metadata::RuntimeMetadataPrefixed;
use node_primitives::Hash;
use parity_codec::Decode;
use ws::Result;

use node_metadata::Module;
use json_rpc::json_req;
use utils::*;

pub mod extrinsic;
pub mod node_metadata;
pub mod utils;
pub mod json_rpc;

#[derive(Serialize, Deserialize, Debug)]
struct JsonBasic {
    jsonrpc: String,
    method: String,
    params: String,
}

#[derive(Debug)]
pub struct Api {
    url : String,
    pub genesis_hash : Option<Hash>,
    pub metadata : Vec<Module>,
}

impl Api {
    pub fn new(url: String) -> Api {
        Api {
            url : url,
            genesis_hash : None,
            metadata: Vec::<Module>::new()
        }
    }

    pub fn init(&mut self) {
        // get genesis hash
        let jsonreq = json_req::chain_get_block_hash();
        let genesis_hash_str = self.get_request(jsonreq.to_string()).unwrap();
        self.genesis_hash = Some(hexstr_to_hash(genesis_hash_str));
        info!("got genesis hash: {:?}", self.genesis_hash.unwrap());

        //get metadata
        let jsonreq = json_req::state_get_metadata();
        let metadata_str = self.get_request(jsonreq.to_string()).unwrap();
        let _unhex = hexstr_to_vec(metadata_str);
        let mut _om = _unhex.as_slice();
        let _meta = RuntimeMetadataPrefixed::decode(&mut _om)
                .expect("runtime metadata decoding to RuntimeMetadataPrefixed failed.");

//        configure::pretty_print(&_meta);
        self.metadata = node_metadata::parse_metadata_into_module_and_call(&_meta)
    }

    // low level access
    pub fn get_request(&self, jsonreq: String) -> Result<String> {
        let (result_in, result_out) = channel();
        json_rpc::get(self.url.clone(), jsonreq.clone(), result_in.clone());

        Ok(result_out.recv().unwrap())
    }

    pub fn get_storage(&self, module: &str, storage_key_name: &str, param: Option<Vec<u8>>) -> Result<String> {
        let keyhash = storage_key_hash(module, storage_key_name, param);

        debug!("with storage key: {}", keyhash);
        let jsonreq = json_req::state_get_storage(&keyhash);
        self.get_request(jsonreq.to_string())
    }

    pub fn send_extrinsic(&self, xthex_prefixed: String) -> Result<Hash> {
        debug!("sending extrinsic: {:?}", xthex_prefixed);

        let jsonreq = json_req::author_submit_and_watch_extrinsic(&xthex_prefixed).to_string();

        let (result_in, result_out) = channel();
        json_rpc::send_extrinsic_and_wait_until_finalized(self.url.clone(),
                                                          jsonreq.clone(),
                                                          result_in.clone());

        Ok(hexstr_to_hash(result_out.recv().unwrap()))
    }

    pub fn subscribe_events(&self, sender: ThreadOut<String>) {
        debug!("subscribing to events");
        let key = storage_key_hash("System", "Events", None);
        let jsonreq = json_req::state_subscribe_storage(&key).to_string();

        let (result_in, result_out) = channel();

        json_rpc::start_event_subscriber(self.url.clone(),
                                         jsonreq.clone(),
                                         result_in.clone());

        loop {
            let res = result_out.recv().unwrap();
            sender.send(res.clone()).unwrap();

/*
            //println!("client >>>> got {}", res);
            let _unhex = hexstr_to_vec(res);
            let mut _er_enc = _unhex.as_slice();
            //let _event = balances::RawEvent::decode(&mut _er_enc2);
            let _events = Vec::<system::EventRecord::<node_runtime::Event>>::decode(&mut _er_enc);
            match _events {
                Some(evts) => {
                    for ev in &evts {
                        println!("decoded: phase {:?} event {:?}", ev.phase, ev.event);
                        sender.send(ev.event.clone()).unwrap();
                    }
                }
                None => println!("couldn't decode event record list")
            }
            //self.result.send(_events).unwrap();
*/
        }
    }
}
