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
	helpers, to_json_req, tungstenite_client::subscription::TungsteniteSubscriptionWrapper,
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
	/// Create a new client with the given url string.
	/// Example url input: "ws://127.0.0.1:9944"
	pub fn new(url: &str, max_attempts: u8) -> Result<Self> {
		let url: Url = Url::parse(url)?;
		Ok(Self { url, max_attempts })
	}

	/// Create a new client with the given address, port and max number of reconnection attempts.
	/// Example input:
	/// - address: "ws://127.0.0.1"
	/// - port: 9944
	pub fn new_with_port(address: &str, port: u32, max_attempts: u8) -> Result<Self> {
		let url = format!("{address}:{port:?}");
		Self::new(&url, max_attempts)
	}

	/// Create a new client with a local address and default Substrate node port.
	pub fn with_default_url(max_attempts: u8) -> Self {
		// This unwrap is safe as is only regards the url parsing, which is tested.
		Self::new("ws://127.0.0.1:9944", max_attempts).unwrap()
	}
}

#[maybe_async::maybe_async(?Send)]
impl Request for TungsteniteRpcClient {
	async fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> Result<R> {
		let json_req = to_json_req(method, params)?;
		let response = self.direct_rpc_request(json_req)?;
		let deserialized_value: R = serde_json::from_str(&response)?;
		Ok(deserialized_value)
	}
}

#[maybe_async::maybe_async(?Send)]
impl Subscribe for TungsteniteRpcClient {
	type Subscription<Notification> = TungsteniteSubscriptionWrapper<Notification> where Notification: DeserializeOwned;

	async fn subscribe<Notification: DeserializeOwned>(
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
		socket.send(Message::Text(json_req))?;

		let msg = read_until_text_message(&mut socket)?;

		trace!("Got get_request_msg {}", msg);
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
	socket.send(Message::Text(json_req))?;

	// Read the first message response - must be the subscription id.
	let msg = read_until_text_message(&mut socket)?;
	let value: Value = serde_json::from_str(&msg)?;

	let subcription_id = match helpers::read_subscription_id(&value) {
		Some(id) => id,
		None => {
			let message = helpers::read_error_message(&value, &msg);
			result_in.send(message)?;
			return Ok(())
		},
	};

	loop {
		let msg = read_until_text_message(&mut socket)?;
		send_message_to_client(result_in.clone(), &msg, &subcription_id)?;
	}
}

pub fn do_reconnect(error: &RpcClientError) -> bool {
	matches!(
		error,
		RpcClientError::SerdeJson(_) | RpcClientError::ConnectionClosed | RpcClientError::Client(_)
	)
}

fn send_message_to_client(
	result_in: ThreadOut<String>,
	message: &str,
	subscription_id: &str,
) -> Result<()> {
	trace!("got on_subscription_msg {}", message);
	let value: Value = serde_json::from_str(message)?;

	if helpers::subscription_id_matches(&value, subscription_id) {
		result_in.send(serde_json::to_string(&value["params"]["result"])?)?;
	}

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
		match socket.read()? {
			Message::Text(s) => break Ok(s),
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn client_new() {
		let port = 9944;
		let address = "ws://127.0.0.1";
		let client = TungsteniteRpcClient::new_with_port(address, port, 1).unwrap();

		let expected_url = Url::parse("ws://127.0.0.1:9944").unwrap();
		assert_eq!(client.url, expected_url);
	}

	#[test]
	fn client_with_default_url() {
		let expected_url = Url::parse("ws://127.0.0.1:9944").unwrap();
		let client = TungsteniteRpcClient::with_default_url(1);

		assert_eq!(client.url, expected_url);
	}
}
