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

use crate::rpc::{Error, Result};
use async_trait::async_trait;
use jsonrpsee::{
	core::client::{Client, ClientT, Subscription, SubscriptionClientT},
	rpc_params,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::value::RawValue;

#[async_trait]
pub trait AsyncClientTrait {
	async fn request<Params: Serialize, R: DeserializeOwned>(
		&self,
		method: &str,
		params: Option<Params>,
	) -> Result<R>;

	async fn subscribe<Params: Serialize, Notif: DeserializeOwned>(
		&self,
		sub: &str,
		params: Option<Params>,
		unsub: &str,
	) -> Result<Subscription<Notif>>;
}

#[async_trait]
impl AsyncClientTrait for Client {
	async fn request<Params: Serialize, R: DeserializeOwned>(
		&self,
		method: &str,
		params: Option<Params>,
	) -> Result<R> {
		let params = params.map_or(rpc_params![], |p| rpc_params![params]);
		ClientT::request(self, method, params)
			.await
			.map_err(|e| Error::Client(Box::new(e)))
	}

	async fn subscribe<Params: Serialize, Notif: DeserializeOwned>(
		&self,
		sub: &str,
		params: Option<Params>,
		unsub: &str,
	) -> Result<Subscription<Notif>> {
		let params = params.map_or(rpc_params![], |p| rpc_params![params]);
		SubscriptionClientT::subscribe::<Box<RawValue>, _>(self, sub, params, unsub)
			.await
			.map_err(|e| Error::Client(Box::new(e)))
	}
}
