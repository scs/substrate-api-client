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
#[macro_use]
extern crate serde_derive;

use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as ThreadOut;
use std::thread;

use metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use node_primitives::Hash;
use parity_codec::Decode;
use ws::{CloseCode, connect, Handler, Handshake, Message, Result, Sender};

use json_req::REQUEST_TRANSFER;
use utils::*;

pub mod extrinsic;
pub mod utils;
pub mod json_req;


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
    //pub metadata : Option<RuntimeMetadataV4>,
}

impl Api {
    pub fn new(url: String) -> Api {
        Api {
            url : url,
            genesis_hash : None,
//            metadata : None,
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
        debug!("decoded: {:?} ", _meta);
        match _meta.1 {
            RuntimeMetadata::V5(_value) => {
                //FIXME: storing metadata in self is problematic because it can't be cloned or synced among threads
                //self.metadata = Some(value);
                debug!("successfully decoded metadata");
            },
            _ => panic!("unsupported metadata"),
        }


/*                    match value.modules {
                        DecodeDifferent::Decoded(mods) => {
                            modules = mods;
                            println!("module0 {:?}", modules[0]);
                        },
                        _ => panic!("unsupported metadata"),
                    }

            println!("-------------------- modules ----------------");
            for module in modules {
                println!("module: {:?}", module.name);
                match module.name {
                    DecodeDifferent::Decoded(name) => {
                        match module.calls {
                            Some(DecodeDifferent::Decoded(calls)) => {
                                println!("calls: {:?}", calls);
                            },
                            _ => println!("ignoring"),
                        }
                        println!("storage: {:?}", module.storage)
                    },
                    _ => println!("ignoring"),
                }
            }
            */
    }

    // low level access
    pub fn get_request(&self, jsonreq: String) -> Result<String> {
        let (result_in, result_out) = channel();
        start_rpc_getter_thread(self.url.clone(),
                                jsonreq.clone(),
                                result_in.clone(),
                                on_get_request_msg);

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
        start_rpc_getter_thread(self.url.clone(),
                                jsonreq.clone(),
                                result_in.clone(),
                                on_extrinsic_msg);

        Ok(hexstr_to_hash(result_out.recv().unwrap()))
    }

    pub fn subscribe_events(&self, sender: ThreadOut<String>) {
        debug!("subscribing to events");
        let key = storage_key_hash("System", "Events", None);
        let jsonreq = json_req::state_subscribe_storage(&key).to_string();

        let (result_in, result_out) = channel();

        start_rpc_getter_thread(self.url.clone(),
                                jsonreq.clone(),
                                result_in.clone(),
                                on_subscription_msg);

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

type OnMessageFn = fn(msg: Message, out: Sender, result: ThreadOut<String>) -> Result<()>;

fn start_rpc_getter_thread(url: String,
                           jsonreq: String,
                           result_in: ThreadOut<String>,
                           on_message_fn: OnMessageFn) {

    let _client = thread::Builder::new()
        .name("client".to_owned())
        .spawn(move || {
            connect(url, |out| {
                GenericGetter {
                    out: out,
                    request: jsonreq.clone(),
                    result: result_in.clone(),
                    on_message_fn: on_message_fn,
                }
            }).unwrap()
        })
        .unwrap();
}

struct GenericGetter {
    out: Sender,
    request: String,
    result: ThreadOut<String>,
    on_message_fn: OnMessageFn,
}

impl Handler for GenericGetter {
    fn on_open(&mut self, _: Handshake) -> Result<()> {

        info!("sending request: {}", self.request);
        self.out.send(self.request.clone()).unwrap();
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        info!("got message");
        debug!("{}", msg);
        (self.on_message_fn)(msg, self.out.clone(), self.result.clone())
    }
}

fn on_get_request_msg(msg: Message, out: Sender, result: ThreadOut<String>) -> Result<()> {
    let retstr = msg.as_text().unwrap();
    let value: serde_json::Value = serde_json::from_str(retstr).unwrap();

    // FIXME: defaulting zo zero can be problematic. better to use Option<String>
    let hexstr = match value["result"].as_str() {
        Some(res) => res.to_string(),
        _ => "0x00".to_string(),
    };

    result.send(hexstr).unwrap();
    out.close(CloseCode::Normal).unwrap();
    Ok(())
}

fn on_subscription_msg(msg: Message, _out: Sender, result: ThreadOut<String>) -> Result<()> {
    let retstr = msg.as_text().unwrap();
    let value: serde_json::Value = serde_json::from_str(retstr).unwrap();
    match value["id"].as_str() {
        Some(_idstr) => { },
        _ => {
            // subscriptions
            debug!("no id field found in response. must be subscription");
            debug!("method: {:?}", value["method"].as_str());
            match value["method"].as_str() {
                Some("state_storage") => {
                    let _changes = &value["params"]["result"]["changes"];
                    let _res_str = _changes[0][1].as_str().unwrap().to_string();
                    result.send(_res_str).unwrap();
                }
                _ => error!("unsupported method"),
            }
        },
    };
    Ok(())
}

fn on_extrinsic_msg(msg: Message, out: Sender, result: ThreadOut<String>) -> Result<()> {
    let retstr = msg.as_text().unwrap();
    let value: serde_json::Value = serde_json::from_str(retstr).unwrap();
    match value["id"].as_str() {
        Some(idstr) => { match idstr.parse::<u32>() {
            Ok(REQUEST_TRANSFER) => {
                match value.get("error") {
                    Some(err) => error!("ERROR: {:?}", err),
                    _ => debug!("no error"),
                }
            },
            Ok(_) => debug!("unknown request id"),
            Err(_) => error!("error assigning request id"),
        }},
        _ => {
            // subscriptions
            debug!("no id field found in response. must be subscription");
            debug!("method: {:?}", value["method"].as_str());
            match value["method"].as_str() {
                Some("author_extrinsicUpdate") => {
                    match value["params"]["result"].as_str() {
                        Some(res) => debug!("author_extrinsicUpdate: {}", res),
                        _ => {
                            debug!("author_extrinsicUpdate: finalized: {}", value["params"]["result"]["finalized"].as_str().unwrap());
                            // return result to calling thread
                            result.send(value["params"]["result"]["finalized"].as_str().unwrap().to_string()).unwrap();
                            // we've reached the end of the flow. return
                            out.close(CloseCode::Normal).unwrap();
                        },
                    }
                }
                _ => error!("unsupported method"),
            }
        },
    };
    Ok(())
}
