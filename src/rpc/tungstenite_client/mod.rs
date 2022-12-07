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

use crate::{
	rpc::{error::Error as RpcClientError, parse_status, result_from_json_response},
	HandleMessage, XtStatus,
};

use log::{debug, error};
use serde_json::Value;
use tungstenite::Message;

pub mod client;

use client::MySocket;

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct GetRequestHandler;

impl HandleMessage for GetRequestHandler {
	type ThreadMessage = String;
	type Error = RpcClientError;
	type Context = MySocket;
	type Result = String;

	fn handle_message(&self, context: &mut Self::Context) -> Result<Self::Result, Self::Error> {
		let msg = read_until_text_message(context)?;
		debug!("Got get_request_msg {}", msg);
		let result_str =
			serde_json::from_str(msg.as_str()).map(|v: Value| v["result"].to_string())?;
		Ok(result_str)
	}
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct SubscriptionHandler {}

impl HandleMessage for SubscriptionHandler {
	type ThreadMessage = String;
	type Error = RpcClientError;
	type Context = MySocket;
	type Result = String;

	fn handle_message(&self, context: &mut Self::Context) -> Result<Self::Result, Self::Error> {
		loop {
			let msg = read_until_text_message(context)?;
			debug!("got on_subscription_msg {}", msg);
			let value: Value = serde_json::from_str(msg.as_str())?;

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
								Some(change_set) => return Ok(change_set.to_string()),
								None => println!("No events happened"),
							};
						},
						Some("chain_finalizedHead") => {
							let head = serde_json::to_string(&value["params"]["result"])?;
							return Ok(head)
						},
						_ => error!("unsupported method"),
					}
				},
			};
		}
	}
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct SubmitOnlyHandler;

impl HandleMessage for SubmitOnlyHandler {
	type ThreadMessage = String;
	type Error = RpcClientError;
	type Context = MySocket;
	type Result = String;

	fn handle_message(&self, context: &mut Self::Context) -> Result<Self::Result, Self::Error> {
		let msg = read_until_text_message(context)?;
		debug!("got msg {}", msg);
		return match result_from_json_response(msg.as_str()) {
			Ok(val) => Ok(val),
			Err(e) => Err(e),
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
	type ThreadMessage = String;
	type Error = RpcClientError;
	type Context = MySocket;
	type Result = String;

	fn handle_message(&self, context: &mut Self::Context) -> Result<Self::Result, Self::Error> {
		loop {
			let msg = read_until_text_message(context)?;
			debug!("receive msg:{:?}", msg);
			match parse_status(msg.as_str()) {
				Ok((xt_status, val)) =>
					if xt_status as u32 >= 10 {
						let error = RpcClientError::Extrinsic(format!(
							"Unexpected extrinsic status: {:?}, stopped watch process prematurely.",
							xt_status
						));
						return Err(error)
					} else if xt_status as u32 >= self.exit_on as u32 {
						return Ok(val.unwrap_or_default())
					},
				Err(e) => return Err(e),
			}
		}
	}
}

pub(crate) fn read_until_text_message(socket: &mut MySocket) -> Result<String, tungstenite::Error> {
	loop {
		match socket.read_message() {
			Ok(Message::Text(s)) => {
				debug!("receive text: {:?}", s);
				break Ok(s)
			},
			Ok(Message::Binary(_)) => {
				debug!("skip binary msg");
			},
			Ok(Message::Ping(_)) => {
				debug!("skip ping msg");
			},
			Ok(Message::Pong(_)) => {
				debug!("skip ping msg");
			},
			Ok(Message::Close(_)) => {
				debug!("skip close msg");
			},
			Ok(Message::Frame(_)) => {
				debug!("skip frame msg");
			},
			Err(e) => break Err(e),
		}
	}
}
