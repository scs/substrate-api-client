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

use ws::{connect, Handler, Sender, Handshake, Result, Message, CloseCode};
use std::{i64, net::SocketAddr};

use substrate_api_client::Client;

#[macro_use]
extern crate clap;
use clap::App;


fn main() {
    let yml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yml).get_matches();

    let cli_protocol = "ws";
    let cli_address  = "127.0.0.1";
    let mut cli_port = "9944";

    if let Some(p) = matches.value_of("wsport") {
        cli_port = p;
    }

    let cli_server = format!("{}://{}:{}", cli_protocol, cli_address, cli_port);

    // Now, instead of a closure, the Factory returns a new instance of our Handler.
    //   connect("ws://127.0.0.1:9944", |out| Client::new(out)).unwrap()
    println!("Connecting to {}", cli_server);
    connect(cli_server, |out| Client::new(out)).unwrap()
}