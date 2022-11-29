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

use async_client::AsyncClientTrait;
use futures::executor::block_on;
use jsonrpsee::core::client::{Client, ClientT, SubscriptionClientT};
use serde_json::value::{RawValue, Value};
use std::sync::Arc;

use crate::rpc::{Error, Result, RpcClient};

mod async_client;

#[derive(Clone)]
pub struct SyncClient(Arc<dyn AsyncClientTrait>);

impl RpcClient for SyncClient {
	fn request(&self, method: &str, params: Value) -> Result<String> {
		block_on(self.0.request(method, params))
	}

	fn send_extrinsic(
		&self,
		xthex_prefixed: String,
		exit_on: crate::XtStatus,
	) -> Result<Option<ac_primitives::Hash>> {
		todo!()
	}
}
