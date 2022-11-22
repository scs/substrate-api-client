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
#[cfg(feature = "ws-client")]
pub use ws_client::WsRpcClient;

#[cfg(feature = "ws-client")]
pub mod ws_client;

pub mod error;
pub mod json_req;

pub use error::*;

use crate::{api::XtStatus, Hash};
use std::sync::mpsc::Sender as ThreadOut;

/// Trait to be implemented by the ws-client for sending rpc requests and extrinsic.
pub trait RpcClient {
	/// Sends a RPC request to the substrate node and returns the answer as string.
	fn get_request(&self, jsonreq: serde_json::Value) -> Result<String>;

	/// Submits ans watches an extrinsic until requested XtStatus and returns the block hash
	/// the extrinsic was included, if XtStatus is InBlock or Finalized.
	fn send_extrinsic(&self, xthex_prefixed: String, exit_on: XtStatus) -> Result<Option<Hash>>;
}

/// Trait to be implemented by the ws-client for subscribing to the substrate node.
pub trait Subscriber {
	fn start_subscriber(&self, json_req: String, result_in: ThreadOut<String>) -> Result<()>;
}
