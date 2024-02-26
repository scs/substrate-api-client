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

use crate::rpc::{Error, Request, Result, RpcParams, Subscribe};
use jsonrpsee::{
	client_transport::ws::{Url, WsTransportClientBuilder},
	core::{
		client::{Client, ClientBuilder, ClientT, Error as JsonrpseeError, SubscriptionClientT},
		traits::ToRpcParams,
	},
};
use serde::de::DeserializeOwned;
use serde_json::value::RawValue;
use std::sync::Arc;

pub use subscription::SubscriptionWrapper;

mod subscription;

#[derive(Clone)]
pub struct JsonrpseeClient {
	inner: Arc<Client>,
}

impl JsonrpseeClient {
	/// Create a new client to a local Substrate node with default port.
	pub async fn with_default_url() -> Result<Self> {
		Self::new("ws://127.0.0.1:9944").await
	}

	/// Create a new client with the given url string.
	/// Example url input: "ws://127.0.0.1:9944"
	pub async fn new(url: &str) -> Result<Self> {
		let parsed_url: Url = url.parse().map_err(|e| Error::Client(Box::new(e)))?;
		let (tx, rx) = WsTransportClientBuilder::default()
			.build(parsed_url)
			.await
			.map_err(|e| Error::Client(Box::new(e)))?;
		let client = ClientBuilder::default()
			.max_buffer_capacity_per_subscription(4096)
			.build_with_tokio(tx, rx);
		Ok(Self { inner: Arc::new(client) })
	}

	/// Create a new client with the given address, port and max number of reconnection attempts.
	/// Example input:
	/// - address: "ws://127.0.0.1"
	/// - port: 9944
	pub async fn new_with_port(address: &str, port: u32) -> Result<Self> {
		let url = format!("{address}:{port:?}");
		Self::new(&url).await
	}

	/// Create a new client with a user-generated Jsonrpsee Client.
	pub fn new_with_client(client: Client) -> Self {
		let inner = Arc::new(client);
		Self { inner }
	}
}

impl JsonrpseeClient {
	/// Checks if the client is connected to the target.
	pub fn is_connected(&self) -> bool {
		self.inner.is_connected()
	}

	/// This is similar to [`Client::on_disconnect`] but it can be used to get
	/// the reason why the client was disconnected but it's not cancel-safe.
	///
	/// The typical use-case is that this method will be called after
	/// [`Client::on_disconnect`] has returned in a "select loop".
	///
	/// # Cancel-safety
	///
	/// This method is not cancel-safe
	pub async fn disconnect_reason(&self) -> JsonrpseeError {
		self.inner.disconnect_reason().await
	}

	/// Completes when the client is disconnected or the client's background task encountered an error.
	/// If the client is already disconnected, the future produced by this method will complete immediately.
	///
	/// # Cancel safety
	///
	/// This method is cancel safe.
	pub async fn on_disconnect(&self) {
		self.inner.on_disconnect().await;
	}
}

#[maybe_async::async_impl(?Send)]
impl Request for JsonrpseeClient {
	async fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> Result<R> {
		self.inner
			.request(method, RpcParamsWrapper(params))
			.await
			.map_err(|e| Error::Client(Box::new(e)))
	}
}

#[maybe_async::async_impl(?Send)]
impl Subscribe for JsonrpseeClient {
	type Subscription<Notification> = SubscriptionWrapper<Notification> where Notification: DeserializeOwned;

	async fn subscribe<Notification: DeserializeOwned>(
		&self,
		sub: &str,
		params: RpcParams,
		unsub: &str,
	) -> Result<Self::Subscription<Notification>> {
		self.inner
			.subscribe(sub, RpcParamsWrapper(params), unsub)
			.await
			.map(|sub| sub.into())
			.map_err(|e| Error::Client(Box::new(e)))
	}
}

struct RpcParamsWrapper(RpcParams);

impl ToRpcParams for RpcParamsWrapper {
	fn to_rpc_params(self) -> core::result::Result<Option<Box<RawValue>>, serde_json::Error> {
		if let Some(json) = self.0.build() {
			RawValue::from_string(json).map(Some)
		} else {
			Ok(None)
		}
	}
}
