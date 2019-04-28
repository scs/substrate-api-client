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

use substrate_api_client::Api;

use std::sync::mpsc::channel;
use std::thread;

fn main() {
    let mut api = Api::new("ws://127.0.0.1:9944".to_string());
    api.init();

    let (events_in, events_out) = channel();
    
    let _eventsubscriber = thread::Builder::new()
            .name("eventsubscriber".to_owned())
            .spawn(move || {
                api.subscribe_events(events_in.clone());
            })
            .unwrap();
    
    loop {
        let event = events_out.recv().unwrap();
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
                    _ => { 
                        println!("ignoring unsupported balances event");
                        },
                }},
            _ => {
                println!("ignoring unsupported module event: {:?}", event)
                },
        }
    }
}