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

use super::{subscription::WsSubscriptionWrapper, HandleMessage};
use crate::{
	rpc::{
		ws_client::{RequestHandler, RpcClient, SubscriptionHandler},
		Request, Result, Subscribe,
	},
	RpcParams,
};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::{
	fmt::Debug,
	sync::mpsc::{channel, Sender as ThreadOut},
	thread,
};
use ws::{connect, Result as WsResult};

#[derive(Debug, Clone)]
pub struct WsRpcClient {
	url: String,
}

impl WsRpcClient {
	pub fn new(url: &str) -> WsRpcClient {
		WsRpcClient { url: url.to_string() }
	}
}

impl Request for WsRpcClient {
	fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> Result<R> {
		let json_req = json!({
			"method": method,
			"params": params.build(),
			"jsonrpc": "2.0",
			"id": "1",
		});
		let response =
			self.direct_rpc_request(json_req.to_string(), RequestHandler::default())??;
		let deserialized_value: R = serde_json::from_str(&response)?;
		Ok(deserialized_value)
	}
}

impl Subscribe for WsRpcClient {
	type Subscription<Notification> = WsSubscriptionWrapper<Notification> where Notification: DeserializeOwned;

	fn subscribe<Notification: DeserializeOwned>(
		&self,
		sub: &str,
		params: RpcParams,
		_unsub: &str,
	) -> Result<Self::Subscription<Notification>> {
		let json_req = json!({
			"method": sub,
			"params": params.build(),
			"jsonrpc": "2.0",
			"id": "1",
		});
		let (result_in, receiver) = channel();
		let subscription = WsSubscriptionWrapper::new(receiver);
		self.start_rpc_client_thread(
			json_req.to_string(),
			result_in,
			SubscriptionHandler::default(),
		)?;
		Ok(subscription)
	}
}

impl WsRpcClient {
	fn start_rpc_client_thread<MessageHandler>(
		&self,
		jsonreq: String,
		result_in: ThreadOut<MessageHandler::ThreadMessage>,
		message_handler: MessageHandler,
	) -> Result<()>
	where
		MessageHandler: HandleMessage + Clone + Send + 'static,
		MessageHandler::ThreadMessage: Send + Sync + Debug,
	{
		let url = self.url.clone();
		let _client =
			thread::Builder::new()
				.name("client".to_owned())
				.spawn(move || -> WsResult<()> {
					connect(url, |out| RpcClient {
						out,
						request: jsonreq.clone(),
						result: result_in.clone(),
						message_handler: message_handler.clone(),
					})
				})?;
		Ok(())
	}

	fn direct_rpc_request<MessageHandler>(
		&self,
		jsonreq: String,
		message_handler: MessageHandler,
	) -> Result<MessageHandler::ThreadMessage>
	where
		MessageHandler: HandleMessage + Clone + Send + 'static,
		MessageHandler::ThreadMessage: Send + Sync + Debug,
	{
		let (result_in, result_out) = channel();
		connect(self.url.as_str(), |out| RpcClient {
			out,
			request: jsonreq.clone(),
			result: result_in.clone(),
			message_handler: message_handler.clone(),
		})?;
		Ok(result_out.recv()?)
	}
}
