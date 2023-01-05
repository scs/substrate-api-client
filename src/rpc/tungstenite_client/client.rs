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
use crate::rpc::{
	to_json_req, tungstenite_client::subscription::TungsteniteSubscriptionWrapper,
	Error as RpcClientError, Request, Result, Subscribe,
};
use ac_primitives::RpcParams;
use log::*;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
	fmt::Debug,
	net::TcpStream,
	sync::mpsc::{channel, Sender as ThreadOut},
	thread,
	thread::sleep,
	time::Duration,
};
use tungstenite::{
	client::connect_with_config, handshake::client::Response, stream::MaybeTlsStream, Message,
	WebSocket,
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
		let response = self.direct_rpc_request(json_req)?;
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
		self.start_rpc_client_thread(json_req, result_in)?;
		let subscription = TungsteniteSubscriptionWrapper::new(receiver);
		Ok(subscription)
	}
}

impl TungsteniteRpcClient {
	fn direct_rpc_request(&self, json_req: String) -> Result<String> {
		let (mut socket, response) = attempt_connection_until(&self.url, self.max_attempts)?;
		debug!("Connected to the server. Response HTTP code: {}", response.status());

		// Send request to server.
		socket.write_message(Message::Text(json_req))?;

		let msg = read_until_text_message(&mut socket)?;

		debug!("Got get_request_msg {}", msg);
		let result_str =
			serde_json::from_str(msg.as_str()).map(|v: Value| v["result"].to_string())?;
		Ok(result_str)
	}

	fn start_rpc_client_thread(
		&self,
		json_req: String,
		result_in: ThreadOut<String>,
	) -> Result<()> {
		let url = self.url.clone();
		let max_attempts = self.max_attempts;
		thread::spawn(move || {
			let mut current_attempt = 0;
			while current_attempt <= max_attempts {
				if let Err(error) =
					subscribe_to_server(&url, max_attempts, json_req.clone(), result_in.clone())
				{
					if !do_reconnect(&error) {
						break
					}
				}
				current_attempt += 1;
			}
		});
		Ok(())
	}
}

fn subscribe_to_server(
	url: &Url,
	max_attempts: u8,
	json_req: String,
	result_in: ThreadOut<String>,
) -> Result<()> {
	let (mut socket, response) = attempt_connection_until(url, max_attempts)?;
	debug!("Connected to the server. Response HTTP code: {}", response.status());

	// Subscribe to server
	socket.write_message(Message::Text(json_req))?;

	loop {
		let msg = read_until_text_message(&mut socket)?;
		send_message_to_client(result_in.clone(), msg.as_str())?;
	}
}

pub fn do_reconnect(error: &RpcClientError) -> bool {
	matches!(
		error,
		RpcClientError::SerdeJson(_) | RpcClientError::ConnectionClosed | RpcClientError::Client(_)
	)
}

fn send_message_to_client(result_in: ThreadOut<String>, message: &str) -> Result<()> {
	debug!("got on_subscription_msg {}", message);
	let value: Value = serde_json::from_str(message)?;

	match value["id"].as_str() {
		Some(_idstr) => {
			warn!("Expected subscription, but received an id response instead: {:?}", value);
		},
		None => {
			let message = serde_json::to_string(&value["params"]["result"])?;
			result_in.send(message)?;
		},
	};
	Ok(())
}

fn attempt_connection_until(url: &Url, max_attempts: u8) -> Result<(MySocket, Response)> {
	let mut current_attempt: u8 = 0;
	while current_attempt <= max_attempts {
		match connect_with_config(url.clone(), None, u8::MAX - 1) {
			Ok((socket, responses)) => return Ok((socket, responses)),
			Err(e) => warn!("Connection attempt failed due to {:?}", e),
		};
		trace!("Trying to reconnect. Current attempt {}", current_attempt);
		sleep(Duration::from_secs(5));
		current_attempt += 1;
	}

	Err(RpcClientError::MaxConnectionAttemptsExceeded)
}

fn read_until_text_message(socket: &mut MySocket) -> Result<String> {
	loop {
		match socket.read_message()? {
			Message::Text(s) => {
				debug!("receive text: {:?}", s);
				break Ok(s)
			},
			Message::Binary(_) => {
				debug!("skip binary msg");
			},
			Message::Ping(_) => {
				debug!("skip ping msg");
			},
			Message::Pong(_) => {
				debug!("skip ping msg");
			},
			Message::Close(_) => break Err(RpcClientError::ConnectionClosed),
			Message::Frame(_) => {
				debug!("skip frame msg");
			},
		}
	}
}
