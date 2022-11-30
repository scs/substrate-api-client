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
use jsonrpsee::core::client::{ClientT, SubscriptionClientT};
use sp_runtime::Serialize;
use std::sync::Arc;

mod async_client;
mod subscription;

#[derive(Clone)]
pub struct SyncClient(Arc<dyn AsyncClientTrait>);

impl Request for SyncClient {
	fn request<Params: Serialize>(&self, method: &str, params: Params) -> Result<String> {
		// Support async: #278
		block_on(self.0.request(method, params))
	}
}

impl Subscribe for SyncClient {
	fn subscribe<Params: Serialize>(
		&self,
		sub: &str,
		params: Params,
		unsub: &str,
	) -> Result<Self::Subscription> {
		block_on(self.0.subscribe(sub, params, unsub))
	}
}
