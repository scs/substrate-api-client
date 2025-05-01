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

use crate::rpc::{helpers, Error as RpcClientError};
use log::*;
use std::{fmt::Debug, sync::mpsc::Sender as ThreadOut};
use ws::{CloseCode, Handler, Handshake, Message, Result as WsResult, Sender};

pub use client::WsRpcClient;
pub use subscription::WsSubscriptionWrapper;

pub type RpcMessage = crate::rpc::Result<String>;

pub mod client;
pub mod subscription;

#[allow(clippy::result_large_err)]
pub(crate) trait HandleMessage {
	type ThreadMessage;
	type Context;

	fn handle_message(&mut self, context: &mut Self::Context) -> WsResult<()>;
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
	MessageHandler::Context: From<MessageContext<MessageHandler::ThreadMessage>>,
{
	fn on_open(&mut self, _: Handshake) -> WsResult<()> {
		trace!("sending request: {}", self.request);
		self.out.send(self.request.clone())?;
		Ok(())
	}

	fn on_message(&mut self, msg: Message) -> WsResult<()> {
		let mut context: MessageHandler::Context = MessageContext {
			out: self.out.clone(),
			request: self.request.clone(),
			result: self.result.clone(),
			msg,
		}
		.into();
		self.message_handler.handle_message(&mut context)
	}
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub(crate) struct RequestHandler;

impl HandleMessage for RequestHandler {
	type ThreadMessage = RpcMessage;
	type Context = MessageContext<Self::ThreadMessage>;

	fn handle_message(&mut self, context: &mut Self::Context) -> WsResult<()> {
		let result = &context.result;
		let out = &context.out;
		let msg = &context.msg;

		out.close(CloseCode::Normal)
			.unwrap_or_else(|_| warn!("Could not close Websocket normally"));

		trace!("Got get_request_msg {}", msg);
		let result_str = serde_json::from_str(msg.as_text()?)
			.map(|v: serde_json::Value| v["result"].to_string())
			.map_err(RpcClientError::SerdeJson);

		result.send(result_str).map_err(|e| Box::new(e).into())
	}
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub(crate) struct SubscriptionHandler {
	subscription_id: Option<String>,
}

impl HandleMessage for SubscriptionHandler {
	type ThreadMessage = String;
	type Context = MessageContext<Self::ThreadMessage>;

	fn handle_message(&mut self, context: &mut Self::Context) -> WsResult<()> {
		let result = &context.result;
		let out = &context.out;
		let msg = &context.msg.as_text()?;

		trace!("got on_subscription_msg {}", msg);
		let value: serde_json::Value = serde_json::from_str(msg).map_err(Box::new)?;

		let send_result = match self.subscription_id.as_ref() {
			Some(id) => handle_subscription_message(result, &value, id),
			None => {
				self.subscription_id = helpers::read_subscription_id(&value);
				if self.subscription_id.is_none() {
					send_error_response(result, &value, msg)
				} else {
					Ok(())
				}
			},
		};

		if let Err(e) = send_result {
			// This may happen if the receiver has unsubscribed.
			trace!("SendError: {:?}. will close ws", e);
			out.close(CloseCode::Normal)?;
		};
		Ok(())
	}
}

fn handle_subscription_message(
	result: &ThreadOut<String>,
	value: &serde_json::Value,
	subscription_id: &str,
) -> Result<(), RpcClientError> {
	if helpers::subscription_id_matches(value, subscription_id) {
		result.send(serde_json::to_string(&value["params"]["result"])?)?;
	}
	Ok(())
}

fn send_error_response(
	result: &ThreadOut<String>,
	value: &serde_json::Value,
	msg: &str,
) -> Result<(), RpcClientError> {
	result.send(helpers::read_error_message(value, msg))?;
	Ok(())
}
