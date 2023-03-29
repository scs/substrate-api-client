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
use crate::rpc::{
	to_json_req,
	ws_client::{MessageContext, RequestHandler, RpcClient, SubscriptionHandler},
	Request, Result, Subscribe,
};
use ac_primitives::RpcParams;
use serde::de::DeserializeOwned;
use std::{
	fmt::Debug,
	sync::mpsc::{channel, Sender as ThreadOut},
	thread,
};
use url::Url;
use ws::{connect, Result as WsResult, Sender as WsSender};

#[derive(Debug, Clone)]
pub struct WsRpcClient {
	url: Url,
}

impl WsRpcClient {
	pub fn new(url: &str) -> Result<Self> {
		Ok(Self { url: Url::parse(url)? })
	}

	pub fn with_default_url() -> Self {
		Self::new("ws://127.0.0.1:9944").unwrap()
	}
}

impl Request for WsRpcClient {
	fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> Result<R> {
		let json_req = to_json_req(method, params)?;
		let response = self.direct_rpc_request(json_req, RequestHandler::default())??;
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
		let json_req = to_json_req(sub, params)?;
		let (result_in, receiver) = channel();
		let sender =
			self.start_rpc_client_thread(json_req, result_in, SubscriptionHandler::default())?;
		let subscription = WsSubscriptionWrapper::new(sender, receiver);
		Ok(subscription)
	}
}

impl WsRpcClient {
	fn start_rpc_client_thread<MessageHandler>(
		&self,
		jsonreq: String,
		result_in: ThreadOut<MessageHandler::ThreadMessage>,
		message_handler: MessageHandler,
	) -> Result<WsSender>
	where
		MessageHandler: HandleMessage + Clone + Send + 'static,
		MessageHandler::ThreadMessage: Send + Sync + Debug,
		MessageHandler::Context: From<MessageContext<MessageHandler::ThreadMessage>>,
	{
		let mut socket = ws::Builder::new().build(move |out| RpcClient {
			out,
			request: jsonreq.clone(),
			result: result_in.clone(),
			message_handler: message_handler.clone(),
		})?;
		socket.connect(self.url.clone())?;
		let handle = socket.broadcaster();

		let _client =
			thread::Builder::new()
				.name("client".to_owned())
				.spawn(move || -> WsResult<()> {
					socket.run()?;
					Ok(())
				})?;

		Ok(handle)
	}

	fn direct_rpc_request<MessageHandler>(
		&self,
		jsonreq: String,
		message_handler: MessageHandler,
	) -> Result<MessageHandler::ThreadMessage>
	where
		MessageHandler: HandleMessage + Clone + Send + 'static,
		MessageHandler::ThreadMessage: Send + Sync + Debug,
		MessageHandler::Context: From<MessageContext<MessageHandler::ThreadMessage>>,
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
