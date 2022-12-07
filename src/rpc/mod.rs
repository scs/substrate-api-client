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

#[cfg(feature = "tungstenite-client")]
pub use tungstenite_client::client::TungsteniteRpcClient;

#[cfg(feature = "tungstenite-client")]
pub mod tungstenite_client;

pub mod error;
pub mod json_req;

pub use error::*;

use crate::api::{FromHexString, XtStatus};
use log::info;
use serde_json::Value;
use std::sync::mpsc::Sender as ThreadOut;

/// Trait to be implemented by the ws-client for sending rpc requests and extrinsic.
pub trait RpcClient {
	/// Sends a RPC request to the substrate node and returns the optional answer as string.
	fn get_request(&self, jsonreq: serde_json::Value) -> Result<Option<String>>;

	/// Submits ans watches an extrinsic until requested XtStatus and returns the block hash
	/// the extrinsic was included, if XtStatus is InBlock or Finalized.
	fn send_extrinsic<Hash: FromHexString>(
		&self,
		xthex_prefixed: String,
		exit_on: XtStatus,
	) -> Result<Option<Hash>>;
}

/// Trait to be implemented by the ws-client for subscribing to the substrate node.
pub trait Subscriber {
	fn start_subscriber(&self, json_req: String, result_in: ThreadOut<String>) -> Result<()>;
}

#[allow(clippy::result_large_err)]
pub trait HandleMessage {
	type ThreadMessage;
	type Error;
	type Context;
	type Result;

	fn handle_message(
		&self,
		context: &mut Self::Context,
	) -> core::result::Result<Self::Result, Self::Error>;
}

pub(crate) fn parse_status(msg: &str) -> Result<(XtStatus, Option<String>)> {
	let value: Value = serde_json::from_str(msg)?;

	if value["error"].as_object().is_some() {
		return Err(into_extrinsic_err(&value))
	}

	if let Some(obj) = value["params"]["result"].as_object() {
		if let Some(hash) = obj.get("finalized") {
			info!("finalized: {:?}", hash);
			return Ok((XtStatus::Finalized, Some(hash.to_string())))
		} else if let Some(hash) = obj.get("inBlock") {
			info!("inBlock: {:?}", hash);
			return Ok((XtStatus::InBlock, Some(hash.to_string())))
		} else if let Some(array) = obj.get("broadcast") {
			info!("broadcast: {:?}", array);
			return Ok((XtStatus::Broadcast, Some(array.to_string())))
		}
	};

	match value["params"]["result"].as_str() {
		Some("ready") => Ok((XtStatus::Ready, None)),
		Some("future") => Ok((XtStatus::Future, None)),
		Some(&_) => Ok((XtStatus::Unknown, None)),
		None => Ok((XtStatus::Unknown, None)),
	}
}

/// Todo: this is the code that was used in `parse_status` Don't we want to just print the
/// error as is instead of introducing our custom format here?
fn into_extrinsic_err(resp_with_err: &Value) -> Error {
	let err_obj = match resp_with_err["error"].as_object() {
		Some(obj) => obj,
		None => return Error::NoErrorInformationFound(format!("{:?}", resp_with_err)),
	};

	let error = err_obj.get("message").map_or_else(|| "", |e| e.as_str().unwrap_or_default());
	let code = err_obj.get("code").map_or_else(|| -1, |c| c.as_i64().unwrap_or_default());
	let details = err_obj.get("data").map_or_else(|| "", |d| d.as_str().unwrap_or_default());

	Error::Extrinsic(format!("extrinsic error code {}: {}: {}", code, error, details))
}

fn result_from_json_response(resp: &str) -> Result<String> {
	let value: Value = serde_json::from_str(resp)?;

	let resp = value["result"].as_str().ok_or_else(|| into_extrinsic_err(&value))?;

	Ok(resp.to_string())
}
