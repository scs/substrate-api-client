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
#![feature(type_alias_enum_variants)]

extern crate substrate_api_client;

use ws::{connect, Handler, Sender, Handshake, Result, Message, CloseCode};
use std::{i64, net::SocketAddr};

use substrate_api_client::{Api, hexstr_to_u256, extrinsic::transfer};

use keyring::AccountKeyring;
use node_primitives::AccountId;
use parity_codec::{Encode, Decode};
use primitive_types::U256;

use std::sync::mpsc::Sender as ThreadOut;
use std::sync::mpsc::channel;
use std::thread;
use std::sync::Mutex;

fn main() {
    //let mut api = Api::new("ws://127.0.0.1:9944".to_string());
    let mut api = Api::new("ws://127.0.0.1:9979".to_string());
    api.init();

    let apim = Mutex::new(api);

    let (events_in, events_out) = channel();

    
    let _eventsubscriber = thread::Builder::new()
            .name("eventsubscriber".to_owned())
            .spawn(move || {
                let _api = apim.lock().unwrap();
                _api.subscribe_events(events_in.clone());
            })
            .unwrap();
    
    while let event = events_out.recv().unwrap() {
        match &event {
            node_runtime::Event::balances(be) => {
                println!(">>>>>>>>>> balances event: {:?}", be);
                match &be {
                    balances::RawEvent::Transfer(transactor, dest, value, fee) => {
                        println!("Transactor: {:?}", transactor);
                        println!("Destination: {:?}", dest);
                        println!("Value: {:?}", value);
                        println!("Fee: {:?}", fee);
                        },
                    _ => { },
                }},
            _ => {
                println!("ignoring event: {:?}", event)
                },
            
        }

    }

}