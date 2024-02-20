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
	/// Create a new client with the given url string.
	/// Example url input: "ws://127.0.0.1:9944"
	pub fn new(url: &str) -> Result<Self> {
		let url = Url::parse(url)?;
		Ok(Self { url })
	}

	/// Create a new client with the given address and port.
	/// Example input:
	/// - address: "ws://127.0.0.1"
	/// - port: 9944
	pub fn new_with_port(address: &str, port: u32) -> Result<Self> {
		let url = format!("{address}:{port:?}");
		Self::new(&url)
	}

	/// Create a new client with a local address and default Substrate node port.
	pub fn with_default_url() -> Self {
		// This unwrap is safe as is only regards the url parsing, which is tested.
		Self::new("ws://127.0.0.1:9944").unwrap()
	}
}

#[maybe_async::maybe_async(?Send)]
impl Request for WsRpcClient {
	async fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> Result<R> {
		let json_req = to_json_req(method, params)?;
		let response = self.direct_rpc_request(json_req, RequestHandler)??;
		let deserialized_value: R = serde_json::from_str(&response)?;
		Ok(deserialized_value)
	}
}

#[maybe_async::maybe_async(?Send)]
impl Subscribe for WsRpcClient {
	type Subscription<Notification> = WsSubscriptionWrapper<Notification> where Notification: DeserializeOwned;

	async fn subscribe<Notification: DeserializeOwned>(
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn client_new() {
		let port = 9944;
		let address = "ws://127.0.0.1";
		let client = WsRpcClient::new_with_port(address, port).unwrap();

		let expected_url = Url::parse("ws://127.0.0.1:9944").unwrap();
		assert_eq!(client.url, expected_url);
	}

	#[test]
	fn client_with_default_url() {
		let expected_url = Url::parse("ws://127.0.0.1:9944").unwrap();
		let client = WsRpcClient::with_default_url();

		assert_eq!(client.url, expected_url);
	}
}
