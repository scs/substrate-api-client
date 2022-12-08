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

use crate::{rpc::error::Error as RpcClientError, HandleMessage};
use client::MySocket;
use log::*;
use serde_json::Value;
use tungstenite::Message;

pub mod client;
pub mod subscription;

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct RequestHandler;

impl HandleMessage for RequestHandler {
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
				Some(_idstr) => {
					warn!(
						"Expected subscription, but received an id response instead: {:?}",
						value
					);
				},
				None => {
					let answer = serde_json::to_string(&value["params"]["result"])?;
					return Ok(answer)
				},
			};
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
