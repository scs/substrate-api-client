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
use sp_core::H256 as Hash;
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
	url: String,
	max_attempts: u8,
}

impl TungsteniteRpcClient {
	pub fn new(url: &str, max_attempts: u8) -> TungsteniteRpcClient {
		TungsteniteRpcClient { url: url.to_string(), max_attempts }
	}

	fn direct_rpc_request<MessageHandler>(
		&self,
		json_req: String,
		message_handler: MessageHandler,
	) -> Result<String>
	where
		MessageHandler: HandleMessage<Error = (RpcClientError, bool), Context = MySocket, Result = String>
			+ Clone
			+ Send
			+ 'static,
		MessageHandler::ThreadMessage: Send + Sync + Debug,
	{
		let url = Url::parse(self.url.as_str()).map_err(|e| RpcClientError::Other(e.into()))?;
		let mut current_attempt: u8 = 1;
		let mut socket: MySocket;

		while current_attempt <= self.max_attempts {
			match connect_with_config(url.clone(), None, 2) {
				Ok(res) => {
					socket = res.0;
					let response = res.1;
					debug!("Connected to the server. Response HTTP code: {}", response.status());
					if socket.can_read() {
						current_attempt = 1;
						let ping = socket.read_message();
						if ping.is_err() {
							error!("failed to read ping message. error: {:?}", ping.unwrap_err());
						} else {
							debug!(
								"read ping message:{:?}. Connected successfully.",
								ping.unwrap()
							);
							let r = match socket.write_message(Message::Text(json_req.clone())) {
								Ok(_) => {
									let r = message_handler.handle_message(&mut socket);
									let _ = socket.close(None);
									r
								},
								Err(e) => {
									let _ = socket.close(None);
									Err((RpcClientError::TungsteniteWebSocket(e), true))
								},
							};
							match r {
								Ok(e) => return Ok(e),
								Err((e, retry)) => {
									error!(
										"failed to send request. error:{:?}, retry:{:?}",
										e, retry
									);
									if retry {
										if current_attempt == self.max_attempts {
											return Err(e)
										}
									} else {
										return Err(e)
									}
								},
							};
						}
					}
					let _ = socket.close(Some(CloseFrame {
						code: CloseCode::Normal,
						reason: Default::default(),
					}));
				},
				Err(e) => {
					error!("failed to connect the server({:?}). error: {:?}", self.url, e);
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
}

impl RpcClientTrait for TungsteniteRpcClient {
	fn get_request(&self, jsonreq: Value) -> Result<Option<String>> {
		self.direct_rpc_request(jsonreq.to_string(), GetRequestHandler::default())
			.map(|v| Some(v))
		// self.direct_rpc_request(jsonreq.to_string(), on_get_request_msg)
	}

	fn send_extrinsic(
		&self,
		xthex_prefixed: String,
		exit_on: XtStatus,
	) -> Result<Option<sp_core::H256>> {
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
		MessageHandler: HandleMessage<Error = (RpcClientError, bool), Context = MySocket>
			+ Clone
			+ Send
			+ 'static,
		MessageHandler::ThreadMessage: Send + Sync + Debug,
		MessageHandler::ThreadMessage: From<MessageHandler::Result>,
	{
		let url = Url::parse(self.url.as_str()).map_err(|e| RpcClientError::Other(e.into()))?;
		let max_attempts = self.max_attempts;

		thread::spawn(move || {
			let mut current_attempt: u8 = 1;
			let mut socket: MySocket;

			while current_attempt <= max_attempts {
				match connect_with_config(url.clone(), None, max_attempts) {
					Ok(res) => {
						socket = res.0;
						let response = res.1;
						debug!(
							"Connected to the server. Response HTTP code: {}",
							response.status()
						);
						match socket.write_message(Message::Text(json_req.clone())) {
							Ok(_) => {},
							Err(e) => {
								error!("write msg error:{:?}", e);
							},
						}
						if socket.can_read() {
							current_attempt = 1;
							// After sending the subscription request, there will be a response(result)
							let msg_from_req = read_until_text_message(&mut socket);
							match msg_from_req {
								Ok(msg_from_req) => {
									debug!("response message: {:?}", msg_from_req);
									loop {
										let msg = read_until_text_message(&mut socket);
										if msg.is_err() {
											error!("err:{:?}", msg.unwrap_err());
											break
										}
										match message_handler.handle_message(&mut socket) {
											Ok(msg) =>
												if let Err(e) = result_in.send(msg.into()) {
													//thread_message.send(result)
													error!("failed to send channel: {:?} ", e);
													return
												},
											Err((e, retry)) => {
												error!("on_subscription_msg: {:?}", e);
												if retry {
													if current_attempt == max_attempts {
														return
													}
												} else {
													return
												}
											},
										}
									}
								},
								Err(e) => {
									error!("response message error:{:?}", e);
								},
							};
						}
						let _ = socket.close(Some(CloseFrame {
							code: CloseCode::Normal,
							reason: Default::default(),
						}));
					},
					Err(e) => {
						error!("failed to connect the server({:?}). error: {:?}", url, e);
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
			error!("max request attempts exceeded");
		});
		Ok(())
	}
}
