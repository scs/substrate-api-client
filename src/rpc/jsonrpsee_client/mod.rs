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
use futures::executor::block_on;
use jsonrpsee::{
	client_transport::ws::{Uri, WsTransportClientBuilder},
	core::{
		client::{Client, ClientBuilder, ClientT, SubscriptionClientT},
		traits::ToRpcParams,
	},
};
use serde::de::DeserializeOwned;
use serde_json::{value::RawValue, Value};
use std::sync::Arc;
use tokio::runtime::Handle;

pub use subscription::SubscriptionWrapper;

mod subscription;

#[derive(Clone)]
pub struct JsonrpseeClient {
	inner: Arc<Client>,
}

impl JsonrpseeClient {
	pub fn new(url: &str) -> Result<Self> {
		block_on(Self::async_new(url))
	}

	pub fn with_default_url() -> Result<Self> {
		Self::new("ws://127.0.0.1:9944")
	}

	pub async fn async_new(url: &str) -> Result<Self> {
		let uri: Uri = url.parse().map_err(|e| Error::Client(Box::new(e)))?;
		let (tx, rx) = WsTransportClientBuilder::default()
			.build(uri)
			.await
			.map_err(|e| Error::Client(Box::new(e)))?;
		let client = ClientBuilder::default()
			.max_buffer_capacity_per_subscription(4096)
			.build_with_tokio(tx, rx);
		Ok(Self { inner: Arc::new(client) })
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

#[maybe_async::sync_impl]
impl Request for JsonrpseeClient {
	fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> Result<R> {
		let handle = Handle::current();
		let client = self.inner.clone();
		let method_string = method.to_string();

		// The inner jsonrpsee client must not deserialize to the `R` value, because the return value must
		// implement `Send`. But we do not want to enforce the `R` value to implement this solely because we need
		// to de-async something. Therefore, the deserialization must happen outside of the newly spawned thread.
		// We need to spawn a new thread because tokio does not allow the blocking of an asynchronous thread:
		// ERROR: Cannot block the current thread from within a runtime.
		// This happens because a function attempted to block the current thread while the thread is being used to drive asynchronous tasks.
		let string_answer: Value = std::thread::spawn(move || {
			handle.block_on(client.request(&method_string, RpcParamsWrapper(params)))
		})
		.join()
		.unwrap()
		.map_err(|e| Error::Client(Box::new(e)))?;

		let deserialized_value: R = serde_json::from_value(string_answer)?;
		Ok(deserialized_value)
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

#[maybe_async::sync_impl(?Send)]
impl Subscribe for JsonrpseeClient {
	type Subscription<Notification> = SubscriptionWrapper<Notification> where Notification: DeserializeOwned;

	fn subscribe<Notification: DeserializeOwned>(
		&self,
		sub: &str,
		params: RpcParams,
		unsub: &str,
	) -> Result<Self::Subscription<Notification>> {
		block_on(self.inner.subscribe(sub, RpcParamsWrapper(params), unsub))
			.map(|sub| sub.into())
			.map_err(|e| Error::Client(Box::new(e)))
	}
}

struct RpcParamsWrapper(RpcParams);

impl ToRpcParams for RpcParamsWrapper {
	fn to_rpc_params(self) -> core::result::Result<Option<Box<RawValue>>, jsonrpsee::core::Error> {
		if let Some(json) = self.0.build() {
			RawValue::from_string(json)
				.map(Some)
				.map_err(jsonrpsee::core::Error::ParseError)
		} else {
			Ok(None)
		}
	}
}
