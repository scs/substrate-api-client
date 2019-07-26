use std::sync::mpsc::Sender as ThreadOut;
use std::thread;

use ws::connect;

use client::*;

mod client;
pub mod json_req;

pub fn get(url: String, json_req: String, result_in: ThreadOut<String>) {
    start_rpc_client_thread(url, json_req, result_in, on_get_request_msg)
}

pub fn send_extrinsic_and_wait_until_finalized(url: String, json_req: String, result_in: ThreadOut<String>) {
    start_rpc_client_thread(url, json_req, result_in, on_extrinsic_msg)
}

pub fn start_event_subscriber(url: String, json_req: String, result_in: ThreadOut<String>) {
    start_rpc_client_thread(url, json_req, result_in, on_subscription_msg)
}

fn start_rpc_client_thread(url: String,
                           jsonreq: String,
                           result_in: ThreadOut<String>,
                           on_message_fn: OnMessageFn) {

    let _client = thread::Builder::new()
        .name("client".to_owned())
        .spawn(move || {
            connect(url, |out| {
                RpcClient {
                    out: out,
                    request: jsonreq.clone(),
                    result: result_in.clone(),
                    on_message_fn: on_message_fn,
                }
            }).unwrap()
        }).unwrap();
}