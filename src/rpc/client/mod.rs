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

#[cfg(not(target_arch = "wasm32"))]
pub mod ws_client;
#[cfg(not(target_arch = "wasm32"))]
pub use ws_client::{start_rpc_client_thread,RpcClient};

#[cfg(target_arch = "wasm32")]
pub mod websys_client;
#[cfg(target_arch = "wasm32")]
pub use websys_client::start_rpc_client_thread;

pub enum ResultE{
    None,
    Close,
    S(String),
    SClose(String) //send and close
}
pub type OnMessageFn = fn(msg: &str) -> ResultE;
use crate::rpc::json_req::REQUEST_TRANSFER;
use log::{debug, error};
pub fn on_get_request_msg(msg: &str) -> ResultE {
    let value: serde_json::Value = serde_json::from_str(msg).unwrap();
    ResultE::SClose(value["result"].to_string())
}

pub fn on_subscription_msg(msg: &str) -> ResultE {
    let value: serde_json::Value = serde_json::from_str(msg).unwrap();
    match value["id"].as_str() {
        Some(_idstr) => {
            ResultE::None
        }
        _ => {
            // subscriptions
            debug!("no id field found in response. must be subscription");
            debug!("method: {:?}", value["method"].as_str());
            match value["method"].as_str() {
                Some("state_storage") => {
                    let _changes = &value["params"]["result"]["changes"];
                    let _res_str = _changes[0][1].as_str().unwrap().to_string();
                    ResultE::S(_res_str)
                }
                _ => {error!("unsupported method");ResultE::None},
            }
        }
    }
}

pub fn on_extrinsic_msg(msg: &str) -> ResultE{
    let value: serde_json::Value = serde_json::from_str(msg).unwrap();
    match value["id"].as_str() {
        Some(idstr) => match idstr.parse::<u32>() {
            Ok(req_id) => match req_id {
                REQUEST_TRANSFER => match value.get("error") {
                    Some(err) => {error!("ERROR: {:?}", err);ResultE::None},
                    _ => {debug!("no error");ResultE::None},
                },
                _ => {debug!("Unknown request id");ResultE::None},
            },
            Err(_) => {error!("error assigning request id");ResultE::None},
        },
        _ => {
            // subscriptions
            debug!("no id field found in response. must be subscription");
            debug!("method: {:?}", value["method"].as_str());
            match value["method"].as_str() {
                Some("author_extrinsicUpdate") => {
                    match value["params"]["result"].as_str() {
                        Some(res) => {debug!("author_extrinsicUpdate: {}", res);ResultE::None},
                        _ => {
                            debug!(
                                "author_extrinsicUpdate: finalized: {}",
                                value["params"]["result"]["finalized"].as_str().unwrap()
                            );
                            // return result to calling thread
                            ResultE::SClose(value["params"]["result"]["finalized"]
                                        .as_str()
                                        .unwrap()
                                        .to_string())
                        }
                    }
                }
                _ => {error!("unsupported method");ResultE::None},
            }
        }
    }
}
