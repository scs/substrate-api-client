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

use crate::rpc::{Request, Result, Subscribe};
use async_client::AsyncClientTrait;
use futures::executor::block_on;
use jsonrpsee::{
	client_transport::ws::{Uri, WsTransportClientBuilder},
	core::client::{ClientBuilder, ClientT, SubscriptionClientT},
};
use serde::{de::DeserializeOwned, Serialize};

pub use subscription::SubscriptionWrapper;

use std::sync::Arc;

mod async_client;
mod subscription;

#[derive(Clone)]
pub struct JsonrpseeClient(Arc<dyn AsyncClientTrait>);

impl JsonrpseeClient {
	pub fn new(url: &str) -> Result<Self> {
		block_on(Self::async_new(url))
	}

	async fn async_new(url: &str) -> Result<Self> {
		let uri: Uri = url.parse()?;
		let (tx, rx) = WsTransportClientBuilder::default().build(uri).await?;
		Ok(ClientBuilder::default()
			.max_notifs_per_subscription(4096)
			.build_with_tokio(tx, rx))
	}
}

impl Default for JsonrpseeClient {
	fn default() -> Self {
		Self::new("ws://127.0.0.1:9944")
	}
}

impl Request for JsonrpseeClient {
	fn request<Params: Serialize, R: DeserializeOwned>(
		&self,
		method: &str,
		params: Option<Params>,
	) -> Result<R> {
		// Support async: #278
		block_on(self.0.request(method, params))
	}
}

impl<Notification> Subscribe for JsonrpseeClient {
	type Subscription = SubscriptionWrapper<Notification>;

	fn subscribe<Params: Serialize, Notif: DeserializeOwned>(
		&self,
		sub: &str,
		params: Option<Params>,
		unsub: &str,
	) -> Result<Self::Subscription<Notification>> {
		block_on(self.0.subscribe(sub, params, unsub))
	}
}
