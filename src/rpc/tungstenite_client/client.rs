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
	json_req,
	rpc::{Error as RpcClientError, Result, RpcClient as RpcClientTrait},
	tungstenite_client::{
		read_until_text_message, GetRequestHandler, SubmitAndWatchHandler, SubmitOnlyHandler,
		SubscriptionHandler,
	},
	FromHexString, HandleMessage, Subscriber, XtStatus,
};
use log::{debug, error, info, warn};
use serde_json::Value;
use std::{
	fmt::Debug, net::TcpStream, sync::mpsc::Sender as ThreadOut, thread, thread::sleep,
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
	pub fn new(url: Url, max_attempts: u8) -> TungsteniteRpcClient {
		TungsteniteRpcClient { url, max_attempts }
	}

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
}

impl RpcClientTrait for TungsteniteRpcClient {
	fn get_request(&self, jsonreq: Value) -> Result<Option<String>> {
		self.direct_rpc_request(jsonreq.to_string(), GetRequestHandler::default())
			.map(Some)
	}

	fn send_extrinsic<Hash: FromHexString>(
		&self,
		xthex_prefixed: String,
		exit_on: XtStatus,
	) -> Result<Option<Hash>> {
		// Todo: Make all variants return a H256: #175.

		let jsonreq = match exit_on {
			XtStatus::SubmitOnly => json_req::author_submit_extrinsic(&xthex_prefixed).to_string(),
			_ => json_req::author_submit_and_watch_extrinsic(&xthex_prefixed).to_string(),
		};
		let response = self.direct_rpc_request(jsonreq, SubmitAndWatchHandler::new(exit_on))?;
		info!("Got response {:?} while waiting for {:?}", response, exit_on);
		if response.is_empty() {
			Ok(None)
		} else {
			Ok(Some(Hash::from_hex(response)?))
		}
	}
}

impl Subscriber for TungsteniteRpcClient {
	fn start_subscriber(&self, json_req: String, result_in: ThreadOut<String>) -> Result<()> {
		self.start_rpc_client_thread(json_req, result_in, SubscriptionHandler::default())
	}
}

impl TungsteniteRpcClient {
	pub fn get(
		&self,
		json_req: String,
		result_in: ThreadOut<<GetRequestHandler as HandleMessage>::ThreadMessage>,
	) -> Result<()> {
		self.start_rpc_client_thread(json_req, result_in, GetRequestHandler::default())
	}

	pub fn send_extrinsic(
		&self,
		json_req: String,
		result_in: ThreadOut<<SubmitOnlyHandler as HandleMessage>::ThreadMessage>,
	) -> Result<()> {
		self.start_rpc_client_thread(json_req, result_in, SubmitOnlyHandler::default())
	}

	pub fn send_extrinsic_until_ready(
		&self,
		json_req: String,
		result_in: ThreadOut<<SubmitAndWatchHandler as HandleMessage>::ThreadMessage>,
	) -> Result<()> {
		self.start_rpc_client_thread(
			json_req,
			result_in,
			SubmitAndWatchHandler::new(XtStatus::Ready),
		)
	}

	pub fn send_extrinsic_and_wait_until_broadcast(
		&self,
		json_req: String,
		result_in: ThreadOut<<SubmitAndWatchHandler as HandleMessage>::ThreadMessage>,
	) -> Result<()> {
		self.start_rpc_client_thread(
			json_req,
			result_in,
			SubmitAndWatchHandler::new(XtStatus::Broadcast),
		)
	}

	pub fn send_extrinsic_and_wait_until_in_block(
		&self,
		json_req: String,
		result_in: ThreadOut<<SubmitAndWatchHandler as HandleMessage>::ThreadMessage>,
	) -> Result<()> {
		self.start_rpc_client_thread(
			json_req,
			result_in,
			SubmitAndWatchHandler::new(XtStatus::InBlock),
		)
	}

	pub fn send_extrinsic_and_wait_until_finalized(
		&self,
		json_req: String,
		result_in: ThreadOut<<SubmitAndWatchHandler as HandleMessage>::ThreadMessage>,
	) -> Result<()> {
		self.start_rpc_client_thread(
			json_req,
			result_in,
			SubmitAndWatchHandler::new(XtStatus::Finalized),
		)
	}

	pub fn start_rpc_client_thread<MessageHandler>(
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

fn close_connection(socket: &mut MySocket) {
	let _ = socket.close(Some(CloseFrame { code: CloseCode::Normal, reason: Default::default() }));
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
						Err(RpcClientError::TungsteniteWebSocket(e)) => {
							error!("tungstenite error:{:?}", e);
							// catch tungstenite::Error, then attempt to reconnect server
							// if reach the maximum attempts, return an error
							close_connection(&mut socket);
							if current_attempt == max_attempts {
								break
							}
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
		warn!(
			"attempt to request after {} sec. current attempt {}",
			5 * current_attempt,
			current_attempt
		);
		sleep(Duration::from_secs((5 * current_attempt) as u64));
		current_attempt += 1;
	}
	Err(RpcClientError::ConnectionAttemptsExceeded)
}
