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

use crate::rpc::Error as RpcClientError;
pub use ac_node_api::{events::EventDetails, StaticEvent};
pub use client::WsRpcClient;
use log::*;
use std::{fmt::Debug, sync::mpsc::Sender as ThreadOut};
use ws::{CloseCode, Handler, Handshake, Message, Sender};

pub mod client;
pub mod subscription;

type RpcResult<T> = Result<T, RpcClientError>;

pub type RpcMessage = RpcResult<String>;

#[allow(clippy::result_large_err)]
pub(crate) trait HandleMessage {
	type ThreadMessage;
	type Error;
	type Context;
	type Result;

	fn handle_message(
		&self,
		context: &mut Self::Context,
	) -> core::result::Result<Self::Result, Self::Error>;
}

// Clippy says request is never used, even though it is..
#[allow(dead_code)]
pub(crate) struct MessageContext<ThreadMessage> {
	pub out: Sender,
	pub request: String,
	pub result: ThreadOut<ThreadMessage>,
	pub msg: Message,
}

#[derive(Debug, Clone)]
pub(crate) struct RpcClient<MessageHandler, ThreadMessage> {
	pub out: Sender,
	pub request: String,
	pub result: ThreadOut<ThreadMessage>,
	pub message_handler: MessageHandler,
}

impl<MessageHandler: HandleMessage> Handler
	for RpcClient<MessageHandler, MessageHandler::ThreadMessage>
where
	MessageHandler::Error: Into<ws::Error>,
	MessageHandler::Context: From<MessageContext<MessageHandler::ThreadMessage>>,
{
	fn on_open(&mut self, _: Handshake) -> Result<(), ws::Error> {
		info!("sending request: {}", self.request);
		self.out.send(self.request.clone())?;
		Ok(())
	}

	fn on_message(&mut self, msg: Message) -> Result<(), ws::Error> {
		let mut context: MessageHandler::Context = MessageContext {
			out: self.out.clone(),
			request: self.request.clone(),
			result: self.result.clone(),
			msg,
		}
		.into();
		self.message_handler
			.handle_message(&mut context)
			.map_err(|e| e.into())
			.map(|_| ())
	}
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub(crate) struct RequestHandler;

impl HandleMessage for RequestHandler {
	type ThreadMessage = RpcMessage;
	type Error = ws::Error;
	type Context = MessageContext<Self::ThreadMessage>;
	type Result = ();

	fn handle_message(&self, context: &mut Self::Context) -> Result<Self::Result, Self::Error> {
		let result = &context.result;
		let out = &context.out;
		let msg = &context.msg;

		out.close(CloseCode::Normal)
			.unwrap_or_else(|_| warn!("Could not close Websocket normally"));

		info!("Got get_request_msg {}", msg);
		let result_str = serde_json::from_str(msg.as_text()?)
			.map(|v: serde_json::Value| v["result"].to_string())
			.map_err(RpcClientError::Serde);

		result
			.send(result_str)
			.map_err(|e| Box::new(RpcClientError::Send(format!("{:?}", e))).into())
	}
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub(crate) struct SubscriptionHandler {}

impl HandleMessage for SubscriptionHandler {
	type ThreadMessage = String;
	type Error = ws::Error;
	type Context = MessageContext<Self::ThreadMessage>;
	type Result = ();

	fn handle_message(&self, context: &mut Self::Context) -> Result<Self::Result, Self::Error> {
		let result = &context.result;
		let out = &context.out;
		let msg = &context.msg;

		info!("got on_subscription_msg {}", msg);
		let value: serde_json::Value =
			serde_json::from_str(msg.as_text()?).map_err(|e| Box::new(RpcClientError::Serde(e)))?;

		match value["id"].as_str() {
			Some(_idstr) => {
				warn!("Expected subscription, but received an id response instead: {:?}", value);
			},
			None => {
				let answer = serde_json::to_string(&value["params"]["result"])
					.map_err(|e| Box::new(RpcClientError::Serde(e)))?;

				if let Err(e) = result.send(answer) {
					// This may happen if the receiver has unsubscribed.
					trace!("SendError: {}. will close ws", e);
					out.close(CloseCode::Normal)?;
				}
			},
		};
		Ok(())
	}
}
