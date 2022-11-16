use ac_node_api::Events;
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
pub use ac_node_api::{events::EventDetails, StaticEvent};

use crate::{
	std::{
		error::Error, json_req, rpc::RpcClientError, Api, ApiResult, FromHexString,
		RpcClient as RpcClientTrait, XtStatus,
	},
	utils,
};
use ac_node_api::DispatchError;
use ac_primitives::ExtrinsicParams;
use log::*;
use serde_json::Value;
use sp_core::Pair;
use sp_runtime::MultiSignature;
use std::{
	fmt::Debug,
	sync::mpsc::{Receiver, SendError, Sender as ThreadOut},
};
use ws::{CloseCode, Error as WsError, Handler, Handshake, Message, Result as WsResult, Sender};

pub use client::WsRpcClient;

pub mod client;

type RpcResult<T> = Result<T, RpcClientError>;

pub type RpcMessage = RpcResult<Option<String>>;

#[allow(clippy::result_large_err)]
pub trait HandleMessage {
	type ThreadMessage;

	fn handle_message(
		&self,
		msg: Message,
		out: Sender,
		result: ThreadOut<Self::ThreadMessage>,
	) -> WsResult<()>;
}

pub struct RpcClient<MessageHandler, ThreadMessage> {
	pub out: Sender,
	pub request: String,
	pub result: ThreadOut<ThreadMessage>,
	pub message_handler: MessageHandler,
}

impl<MessageHandler: HandleMessage> Handler
	for RpcClient<MessageHandler, MessageHandler::ThreadMessage>
{
	fn on_open(&mut self, _: Handshake) -> WsResult<()> {
		info!("sending request: {}", self.request);
		self.out.send(self.request.clone())?;
		Ok(())
	}

	fn on_message(&mut self, msg: Message) -> WsResult<()> {
		self.message_handler.handle_message(msg, self.out.clone(), self.result.clone())
	}
}

#[allow(clippy::result_large_err)]
pub trait Subscriber {
	fn start_subscriber(
		&self,
		json_req: String,
		result_in: ThreadOut<String>,
	) -> Result<(), WsError>;
}

impl<P, Params> Api<P, WsRpcClient, Params>
where
	Params: ExtrinsicParams,
{
	pub fn default_with_url(url: &str) -> ApiResult<Self> {
		let client = WsRpcClient::new(url);
		Self::new(client)
	}
}

impl<P, Client, Params> Api<P, Client, Params>
where
	P: Pair,
	MultiSignature: From<P::Signature>,
	Client: RpcClientTrait + Subscriber,
	Params: ExtrinsicParams,
{
	pub fn subscribe_events(&self, sender: ThreadOut<String>) -> ApiResult<()> {
		debug!("subscribing to events");
		let key = utils::storage_key("System", "Events");
		let jsonreq = json_req::state_subscribe_storage(vec![key]).to_string();
		self.client.start_subscriber(jsonreq, sender).map_err(|e| e.into())
	}

	pub fn subscribe_finalized_heads(&self, sender: ThreadOut<String>) -> ApiResult<()> {
		debug!("subscribing to finalized heads");
		let jsonreq = json_req::chain_subscribe_finalized_heads().to_string();
		self.client.start_subscriber(jsonreq, sender).map_err(|e| e.into())
	}

	pub fn wait_for_event<Ev: StaticEvent>(&self, receiver: &Receiver<String>) -> ApiResult<Ev> {
		let maybe_event_details = self.wait_for_event_details::<Ev>(receiver)?;
		maybe_event_details
			.as_event()?
			.ok_or(Error::Other("Could not find the specific event".into()))
	}

	pub fn wait_for_event_details<Ev: StaticEvent>(
		&self,
		receiver: &Receiver<String>,
	) -> ApiResult<EventDetails> {
		loop {
			let events_str = receiver.recv()?;
			let event_bytes = Vec::from_hex(events_str)?;
			let events = Events::new(self.metadata.clone(), Default::default(), event_bytes);

			for maybe_event_details in events.iter() {
				let event_details = maybe_event_details?;

				// Check for failed xt and return as Dispatch Error in case we find one.
				// Careful - this reports the first one encountered. This event may belong to another extrinsic
				// than the one that is being waited for.
				if extrinsic_has_failed(&event_details) {
					let dispatch_error =
						DispatchError::decode_from(event_details.field_bytes(), &self.metadata);
					return Err(Error::Dispatch(dispatch_error))
				}

				let event_metadata = event_details.event_metadata();
				trace!(
					"Found extrinsic: {:?}, {:?}",
					event_metadata.pallet(),
					event_metadata.event()
				);
				if event_metadata.pallet() == Ev::PALLET && event_metadata.event() == Ev::EVENT {
					return Ok(event_details)
				} else {
					trace!("Not the event we are looking for, skipping.")
				}
			}
		}
	}
}

fn extrinsic_has_failed(event_details: &EventDetails) -> bool {
	event_details.pallet_name() == "System" && event_details.variant_name() == "ExtrinsicFailed"
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]

pub struct GetRequestHandler;

impl HandleMessage for GetRequestHandler {
	type ThreadMessage = RpcMessage;

	fn handle_message(
		&self,
		msg: Message,
		out: Sender,
		result: ThreadOut<Self::ThreadMessage>,
	) -> WsResult<()> {
		out.close(CloseCode::Normal)
			.unwrap_or_else(|_| warn!("Could not close Websocket normally"));

		info!("Got get_request_msg {}", msg);
		let result_str = serde_json::from_str(msg.as_text()?)
			.map(|v: serde_json::Value| Some(v["result"].to_string()))
			.map_err(RpcClientError::Serde);

		result
			.send(result_str)
			.map_err(|e| Box::new(RpcClientError::Send(format!("{:?}", e))).into())
	}
}
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct SubscriptionHandler {}
impl HandleMessage for SubscriptionHandler {
	type ThreadMessage = String;

	fn handle_message(
		&self,
		msg: Message,
		out: Sender,
		result: ThreadOut<Self::ThreadMessage>,
	) -> WsResult<()> {
		info!("got on_subscription_msg {}", msg);
		let value: serde_json::Value =
			serde_json::from_str(msg.as_text()?).map_err(|e| Box::new(RpcClientError::Serde(e)))?;

		match value["id"].as_str() {
			Some(_idstr) => {},
			_ => {
				// subscriptions
				debug!("no id field found in response. must be subscription");
				debug!("method: {:?}", value["method"].as_str());
				match value["method"].as_str() {
					Some("state_storage") => {
						let changes = &value["params"]["result"]["changes"];
						match changes[0][1].as_str() {
							Some(change_set) => {
								if let Err(SendError(e)) = result.send(change_set.to_owned()) {
									debug!("SendError: {:?}. will close ws", e);
									out.close(CloseCode::Normal)?;
								}
							},
							None => println!("No events happened"),
						};
					},
					Some("chain_finalizedHead") => {
						let head = serde_json::to_string(&value["params"]["result"])
							.map_err(|e| Box::new(RpcClientError::Serde(e)))?;

						if let Err(e) = result.send(head) {
							debug!("SendError: {}. will close ws", e);
							out.close(CloseCode::Normal)?;
						}
					},
					_ => error!("unsupported method"),
				}
			},
		};
		Ok(())
	}
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct SubmitOnlyHandler;

impl HandleMessage for SubmitOnlyHandler {
	type ThreadMessage = RpcMessage;

	fn handle_message(
		&self,
		msg: Message,
		out: Sender,
		result: ThreadOut<Self::ThreadMessage>,
	) -> WsResult<()> {
		let retstr = msg.as_text()?;
		debug!("got msg {}", retstr);
		match result_from_json_response(retstr) {
			Ok(val) => end_process(out, result, Ok(Some(val))),
			Err(e) => end_process(out, result, Err(e)),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubmitAndWatchHandler {
	exit_on: XtStatus,
}
impl SubmitAndWatchHandler {
	pub fn new(exit_on: XtStatus) -> Self {
		Self { exit_on }
	}
}

impl HandleMessage for SubmitAndWatchHandler {
	type ThreadMessage = RpcMessage;

	fn handle_message(
		&self,
		msg: Message,
		out: Sender,
		result: ThreadOut<Self::ThreadMessage>,
	) -> WsResult<()> {
		let return_string = msg.as_text()?;
		debug!("got msg {}", return_string);
		match parse_status(return_string) {
			Ok((xt_status, val)) => {
				if xt_status as u32 >= 10 {
					let error = RpcClientError::Extrinsic(format!(
						"Unexpected extrinsic status: {:?}, stopped watch process prematurely.",
						xt_status
					));
					end_process(out, result, Err(error))?;
				} else if xt_status as u32 >= self.exit_on as u32 {
					end_process(out, result, Ok(val))?;
				}
				Ok(())
			},
			Err(e) => end_process(out, result, Err(e)),
		}
	}
}

#[allow(clippy::result_large_err)]
fn end_process<ThreadMessage: Send + Sync + Debug>(
	out: Sender,
	result: ThreadOut<ThreadMessage>,
	value: ThreadMessage,
) -> WsResult<()> {
	// return result to calling thread
	debug!("Thread end result :{:?} value:{:?}", result, value);

	out.close(CloseCode::Normal)
		.unwrap_or_else(|_| warn!("Could not close WebSocket normally"));

	result
		.send(value)
		.map_err(|e| Box::new(RpcClientError::Send(format!("{:?}", e))).into())
}

fn parse_status(msg: &str) -> RpcResult<(XtStatus, Option<String>)> {
	let value: serde_json::Value = serde_json::from_str(msg)?;

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
fn into_extrinsic_err(resp_with_err: &Value) -> RpcClientError {
	let err_obj = match resp_with_err["error"].as_object() {
		Some(obj) => obj,
		None => return RpcClientError::NoErrorInformationFound(format!("{:?}", resp_with_err)),
	};

	let error = err_obj.get("message").map_or_else(|| "", |e| e.as_str().unwrap_or_default());
	let code = err_obj.get("code").map_or_else(|| -1, |c| c.as_i64().unwrap_or_default());
	let details = err_obj.get("data").map_or_else(|| "", |d| d.as_str().unwrap_or_default());

	RpcClientError::Extrinsic(format!("extrinsic error code {}: {}: {}", code, error, details))
}

fn result_from_json_response(resp: &str) -> RpcResult<String> {
	let value: serde_json::Value = serde_json::from_str(resp)?;

	let resp = value["result"].as_str().ok_or_else(|| into_extrinsic_err(&value))?;

	Ok(resp.to_string())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::rpc::RpcClientError;
	use std::{assert_matches::assert_matches, fmt::Debug};

	fn assert_extrinsic_err<T: Debug>(result: Result<T, RpcClientError>, msg: &str) {
		assert_matches!(result.unwrap_err(), RpcClientError::Extrinsic(
			m,
		) if m == msg)
	}

	#[test]
	fn result_from_json_response_works() {
		let msg = r#"{"jsonrpc":"2.0","result":"0xe7640c3e8ba8d10ed7fed07118edb0bfe2d765d3ea2f3a5f6cf781ae3237788f","id":"3"}"#;

		assert_eq!(
			result_from_json_response(msg).unwrap(),
			"0xe7640c3e8ba8d10ed7fed07118edb0bfe2d765d3ea2f3a5f6cf781ae3237788f"
		);
	}

	#[test]
	fn result_from_json_response_errs_on_error_response() {
		let _err_raw =
			r#"{"code":-32602,"message":"Invalid params: invalid hex character: h, at 284."}"#;

		let err_msg = format!(
			"extrinsic error code {}: {}: {}",
			-32602, "Invalid params: invalid hex character: h, at 284.", ""
		);

		let msg = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params: invalid hex character: h, at 284."},"id":"3"}"#;

		assert_extrinsic_err(result_from_json_response(msg), &err_msg)
	}

	#[test]
	fn extrinsic_status_parsed_correctly() {
		let msg = "{\"jsonrpc\":\"2.0\",\"result\":7185,\"id\":\"3\"}";
		assert_eq!(parse_status(msg).unwrap(), (XtStatus::Unknown, None));

		let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"author_extrinsicUpdate\",\"params\":{\"result\":\"ready\",\"subscription\":7185}}";
		assert_eq!(parse_status(msg).unwrap(), (XtStatus::Ready, None));

		let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"author_extrinsicUpdate\",\"params\":{\"result\":{\"broadcast\":[\"QmfSF4VYWNqNf5KYHpDEdY8Rt1nPUgSkMweDkYzhSWirGY\",\"Qmchhx9SRFeNvqjUK4ZVQ9jH4zhARFkutf9KhbbAmZWBLx\",\"QmQJAqr98EF1X3YfjVKNwQUG9RryqX4Hv33RqGChbz3Ncg\"]},\"subscription\":232}}";
		assert_eq!(
            parse_status(msg).unwrap(),
            (
                XtStatus::Broadcast,
                Some(
                    "[\"QmfSF4VYWNqNf5KYHpDEdY8Rt1nPUgSkMweDkYzhSWirGY\",\"Qmchhx9SRFeNvqjUK4ZVQ9jH4zhARFkutf9KhbbAmZWBLx\",\"QmQJAqr98EF1X3YfjVKNwQUG9RryqX4Hv33RqGChbz3Ncg\"]"
                        .to_string()
                )
            )
        );

		let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"author_extrinsicUpdate\",\"params\":{\"result\":{\"inBlock\":\"0x3104d362365ff5ddb61845e1de441b56c6722e94c1aee362f8aa8ba75bd7a3aa\"},\"subscription\":232}}";
		assert_eq!(
			parse_status(msg).unwrap(),
			(
				XtStatus::InBlock,
				Some(
					"\"0x3104d362365ff5ddb61845e1de441b56c6722e94c1aee362f8aa8ba75bd7a3aa\""
						.to_string()
				)
			)
		);

		let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"author_extrinsicUpdate\",\"params\":{\"result\":{\"finalized\":\"0x934385b11c483498e2b5bca64c2e8ef76ad6c74d3372a05595d3a50caf758d52\"},\"subscription\":7185}}";
		assert_eq!(
			parse_status(msg).unwrap(),
			(
				XtStatus::Finalized,
				Some(
					"\"0x934385b11c483498e2b5bca64c2e8ef76ad6c74d3372a05595d3a50caf758d52\""
						.to_string()
				)
			)
		);

		let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"author_extrinsicUpdate\",\"params\":{\"result\":\"future\",\"subscription\":2}}";
		assert_eq!(parse_status(msg).unwrap(), (XtStatus::Future, None));

		let msg = "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":-32700,\"message\":\"Parse error\"},\"id\":null}";
		assert_extrinsic_err(parse_status(msg), "extrinsic error code -32700: Parse error: ");

		let msg = "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":1010,\"message\":\"Invalid Transaction\",\"data\":\"Bad Signature\"},\"id\":\"4\"}";
		assert_extrinsic_err(
			parse_status(msg),
			"extrinsic error code 1010: Invalid Transaction: Bad Signature",
		);

		let msg = "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":1001,\"message\":\"Extrinsic has invalid format.\"},\"id\":\"0\"}";
		assert_extrinsic_err(
			parse_status(msg),
			"extrinsic error code 1001: Extrinsic has invalid format.: ",
		);

		let msg = r#"{"jsonrpc":"2.0","error":{"code":1002,"message":"Verification Error: Execution(Wasmi(Trap(Trap { kind: Unreachable })))","data":"RuntimeApi(\"Execution(Wasmi(Trap(Trap { kind: Unreachable })))\")"},"id":"3"}"#;
		assert_extrinsic_err(
            parse_status(msg),
            "extrinsic error code 1002: Verification Error: Execution(Wasmi(Trap(Trap { kind: Unreachable }))): RuntimeApi(\"Execution(Wasmi(Trap(Trap { kind: Unreachable })))\")"
        );
	}
}
