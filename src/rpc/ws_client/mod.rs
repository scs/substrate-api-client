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
use log::*;
use std::{
	fmt::Debug,
	sync::mpsc::{SendError, Sender as ThreadOut},
};
use ws::{CloseCode, Handler, Handshake, Message, Result as WsResult, Sender};

pub use ac_node_api::{events::EventDetails, StaticEvent};
pub use client::WsRpcClient;

pub mod client;
pub mod subscription;

type RpcResult<T> = Result<T, RpcClientError>;

pub type RpcMessage = RpcResult<String>;

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

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct RequestHandler;

impl HandleMessage for RequestHandler {
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
			.map(|v: serde_json::Value| v["result"].to_string())
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
				debug!("method: {:?}", value["method"].as_str());
				match value["method"].as_str() {
					Some("state_storage") => {
						let changes = &value["params"]["result"]["changes"];
						match changes[0][1].as_str() {
							Some(change_set) => {
								if let Err(SendError(e)) = result.send(change_set.to_owned()) {
									// This may happen if the receiver has unsubscribed.
									trace!("SendError: {:?}. will close ws", e);
									out.close(CloseCode::Normal)?;
								}
							},
							None => println!("No events happened"),
						};
					},
					Some("chain_finalizedHead") | Some("author_extrinsicUpdate") => {
						let answer = serde_json::to_string(&value["params"]["result"])
							.map_err(|e| Box::new(RpcClientError::Serde(e)))?;

						if let Err(e) = result.send(answer) {
							// This may happen if the receiver has unsubscribed.
							trace!("SendError: {}. will close ws", e);
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
