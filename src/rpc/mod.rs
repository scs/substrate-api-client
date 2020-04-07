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

use std::sync::mpsc::Sender as ThreadOut;
use std::thread;

use ws::connect;

use client::*;
pub use client::XtStatus;

mod client;
pub mod json_req;

pub fn get(url: String, json_req: String, result_in: ThreadOut<String>) {
    start_rpc_client_thread(url, json_req, result_in, on_get_request_msg)
}

pub fn send_extrinsic(
    url: String,
    json_req: String,
    result_in: ThreadOut<String>,
) {
    start_rpc_client_thread(url, json_req, result_in, on_extrinsic_msg_until_ready)
}

pub fn send_extrinsic_and_wait_until_finalized(
    url: String,
    json_req: String,
    result_in: ThreadOut<String>,
) {
    start_rpc_client_thread(url, json_req, result_in, on_extrinsic_msg_until_finalized)
}

pub fn start_event_subscriber(url: String, json_req: String, result_in: ThreadOut<String>) {
    start_rpc_client_thread(url, json_req, result_in, on_subscription_msg)
}

fn start_rpc_client_thread(
    url: String,
    jsonreq: String,
    result_in: ThreadOut<String>,
    on_message_fn: OnMessageFn,
) {
    let _client = thread::Builder::new()
        .name("client".to_owned())
        .spawn(move || {
            connect(url, |out| RpcClient {
                out,
                request: jsonreq.clone(),
                result: result_in.clone(),
                on_message_fn,
            })
            .unwrap()
        })
        .unwrap();
}
