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
	rpc::{to_json_req, Error as RpcClientError, Result},
	tungstenite_client::{
		read_until_text_message, subscription::TungsteniteSubscriptionWrapper, RequestHandler,
		SubscriptionHandler,
	},
	HandleMessage, Request, Subscribe,
};
use ac_primitives::RpcParams;
use log::*;
use serde::de::DeserializeOwned;
use std::{
	fmt::Debug,
	net::TcpStream,
	sync::mpsc::{channel, Sender as ThreadOut},
	thread,
	thread::sleep,
	time::Duration,
};
use tungstenite::{
	client::connect_with_config,
	protocol::{frame::coding::CloseCode, CloseFrame},
	stream::MaybeTlsStream,
	Message, WebSocket,
};
use url::Url;

pub(crate) type MySocket = WebSocket<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Clone)]
pub struct TungsteniteRpcClient {
	url: Url,
	max_attempts: u8,
}

impl TungsteniteRpcClient {
	pub fn new(url: &str, max_attempts: u8) -> Result<Self> {
		Ok(Self { url: Url::parse(url)?, max_attempts })
	}

	pub fn with_default_url(max_attempts: u8) -> Self {
		Self::new("ws://127.0.0.1:9944", max_attempts).unwrap()
	}
}

impl Request for TungsteniteRpcClient {
	fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> Result<R> {
		let json_req = to_json_req(method, params)?;
		let response = self.direct_rpc_request(json_req, RequestHandler::default())?;
		let deserialized_value: R = serde_json::from_str(&response)?;
		Ok(deserialized_value)
	}
}

impl Subscribe for TungsteniteRpcClient {
	type Subscription<Notification> = TungsteniteSubscriptionWrapper<Notification> where Notification: DeserializeOwned;

	fn subscribe<Notification: DeserializeOwned>(
		&self,
		sub: &str,
		params: RpcParams,
		_unsub: &str,
	) -> Result<Self::Subscription<Notification>> {
		let json_req = to_json_req(sub, params)?;
		let (result_in, receiver) = channel();
		self.start_rpc_client_thread(json_req, result_in, SubscriptionHandler::default())?;
		let subscription = TungsteniteSubscriptionWrapper::new(receiver);
		Ok(subscription)
	}
}

impl TungsteniteRpcClient {
	fn direct_rpc_request<MessageHandler>(
		&self,
		json_req: String,
		message_handler: MessageHandler,
	) -> Result<String>
	where
		MessageHandler: HandleMessage<Error = RpcClientError, Context = MySocket, Result = String>
			+ Clone
			+ Send
			+ 'static,
		MessageHandler::ThreadMessage: Send + Sync + Debug,
	{
		connect_to_server(
			self.url.clone(),
			self.max_attempts,
			None,
			|socket| -> Result<MessageHandler::Result> {
				match socket.write_message(Message::Text(json_req.clone())) {
					Ok(_) => message_handler.handle_message(socket),
					Err(e) => {
						error!("failed to send request. error:{:?}", e,);
						Err(e.into())
					},
				}
			},
		)
	}

	fn start_rpc_client_thread<MessageHandler>(
		&self,
		json_req: String,
		result_in: ThreadOut<MessageHandler::ThreadMessage>,
		message_handler: MessageHandler,
	) -> Result<()>
	where
		MessageHandler:
			HandleMessage<Error = RpcClientError, Context = MySocket> + Clone + Send + 'static,
		MessageHandler::ThreadMessage: Send + Sync + Debug,
		MessageHandler::ThreadMessage: From<MessageHandler::Result>,
	{
		let url = self.url.clone();
		let max_attempts = self.max_attempts;

		thread::spawn(move || {
			let _ = connect_to_server(
				url,
				max_attempts,
				Some(json_req),
				|socket| -> Result<MessageHandler::Result> {
					loop {
						match message_handler.handle_message(socket) {
							Ok(msg) =>
								if let Err(e) = result_in.send(msg.into()) {
									error!("failed to send channel: {:?} ", e);
									return Err(RpcClientError::Send(format!("{:?}", e)))
								},
							Err(e) => return Err(e),
						}
					}
				},
			);
		});
		Ok(())
	}
}

fn check_connection(socket: &mut MySocket, request: Option<String>) -> bool {
	return if let Some(json_req) = request {
		match socket.write_message(Message::Text(json_req)) {
			Err(e) => {
				error!("write msg error:{:?}", e);
				false
			},
			Ok(_) => {
				// After sending request(subscription), there will be a response(result)
				match read_until_text_message(socket) {
					Ok(msg_from_req) => {
						debug!("response message: {:?}", msg_from_req);
						true
					},
					Err(e) => {
						error!("response message error:{:?}", e);
						false
					},
				}
			},
		}
	} else {
		match socket.read_message() {
			Ok(ping) => {
				debug!("read ping message:{:?}. Connected successfully.", ping);
				true
			},
			Err(err) => {
				error!("failed to read ping message. error: {:?}", err);
				false
			},
		}
	}
}

fn connect_to_server<T, F: Fn(&mut MySocket) -> Result<T>>(
	url: url::Url,
	max_attempts: u8,
	subscription_req: Option<String>,
	handle_message: F,
) -> Result<T> {
	let mut current_attempt: u8 = 1;
	while current_attempt <= max_attempts {
		match connect_with_config(url.clone(), None, u8::MAX - 1) {
			Err(e) => {
				error!("failed to connect the server({:?}). error: {:?}", url, e);
			},
			Ok(res) => {
				let mut socket = res.0;
				debug!("Connected to the server. Response HTTP code: {}", res.1.status());
				if check_connection(&mut socket, subscription_req.clone()) {
					current_attempt = 1; // reset the value once connected successfully
					match handle_message(&mut socket) {
						Err(RpcClientError::Client(e)) => {
							// catch tungstenite::Error, then attempt to reconnect server
							// if reach the maximum attempts, return an error
							close_connection(&mut socket);
							if current_attempt == max_attempts {
								error!(
									"Connection error:{:?}, max retry attempts ({:?}) reached, closing connection.",
									e, max_attempts
								);
								break
							}
							warn!("Connection error:{:?}, trying to reconnect", e);
						},
						Err(e) => {
							// catch other error(not tungstenite error), exit function
							error!("handle message error: {:?}", e);
							close_connection(&mut socket);
							return Err(e)
						},
						Ok(t) => {
							close_connection(&mut socket);
							return Ok(t)
						},
					};
				}
			},
		};
		trace!(
			"attempt to request after {} sec. current attempt {}",
			5 * current_attempt,
			current_attempt
		);
		sleep(Duration::from_secs((5 * current_attempt) as u64));
		current_attempt += 1;
	}
	Err(RpcClientError::ConnectionAttemptsExceeded)
}

fn close_connection(socket: &mut MySocket) {
	let _ = socket.close(Some(CloseFrame { code: CloseCode::Normal, reason: Default::default() }));
}
