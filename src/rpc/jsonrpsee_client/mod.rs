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

use crate::rpc::{Error, Request, Result, Subscribe};
use futures::executor::block_on;
use jsonrpsee::{
	client_transport::ws::{Uri, WsTransportClientBuilder},
	core::client::{Client, ClientBuilder, ClientT, SubscriptionClientT},
	rpc_params,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::value::RawValue;

pub use subscription::SubscriptionWrapper;

mod subscription;

pub struct JsonrpseeClient {
	inner: Client,
}

impl JsonrpseeClient {
	pub fn new(url: &str) -> Result<Self> {
		block_on(Self::async_new(url))
	}

	fn with_default_url() -> Result<Self> {
		Self::new("ws://127.0.0.1:9944")
	}

	async fn async_new(url: &str) -> Result<Self> {
		let uri: Uri = url.parse().map_err(|e| Error::Client(Box::new(e)))?;
		let (tx, rx) = WsTransportClientBuilder::default()
			.build(uri)
			.await
			.map_err(|e| Error::Client(Box::new(e)))?;
		Ok(Self {
			inner: ClientBuilder::default()
				.max_notifs_per_subscription(4096)
				.build_with_tokio(tx, rx),
		})
	}
}

impl Request for JsonrpseeClient {
	fn request<Params: Serialize, R: DeserializeOwned>(
		&self,
		method: &str,
		params: Option<Params>,
	) -> Result<R> {
		let params = params.map_or(rpc_params![], |p| rpc_params![params]);
		// Support async: #278
		block_on(self.inner.request(method, params)).map_err(|e| Error::Client(Box::new(e)))
	}
}

impl Subscribe for JsonrpseeClient {
	fn subscribe<Params: Serialize, Notification: DeserializeOwned>(
		&self,
		sub: &str,
		params: Option<Params>,
		unsub: &str,
	) -> Result<SubscriptionWrapper<Notification>> {
		let params = params.map_or(rpc_params![], |p| rpc_params![params]);
		block_on(self.inner.subscribe(sub, params, unsub)).map_err(|e| Error::Client(Box::new(e)))

		// 		SubscriptionClientT::subscribe::<Box<RawValue>, _>(self, sub, params, unsub)
		// 			.await
		// 			.map_err(|e| Error::Client(Box::new(e)))
	}
}
