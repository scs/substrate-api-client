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
		client::{Client, ClientBuilder, ClientT, SubscriptionClientT},
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
	pub async fn with_default_url() -> Result<Self> {
		Self::new("ws://127.0.0.1:9944").await
	}

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
