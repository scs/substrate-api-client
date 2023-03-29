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

use crate::rpc::{Request, Result};
use ac_primitives::RpcParams;
use serde::de::DeserializeOwned;
use std::{collections::HashMap, sync::RwLock};

type MethodKey = String;
type SerializedValue = String;

#[derive(Debug)]
pub struct RpcClientMock {
	pub state: RwLock<HashMap<MethodKey, SerializedValue>>,
}

impl RpcClientMock {
	pub fn new(state: HashMap<MethodKey, SerializedValue>) -> Self {
		Self { state: RwLock::new(state) }
	}

	pub fn update_entry(&self, key: MethodKey, value: SerializedValue) {
		let mut lock = self.state.write().unwrap();
		lock.insert(key, value);
	}
}

impl Request for RpcClientMock {
	fn request<R: DeserializeOwned>(&self, method: &str, _params: RpcParams) -> Result<R> {
		let lock = self.state.read().unwrap();
		let response = lock.get(method).unwrap();
		let deserialized_value: R = serde_json::from_str(response).unwrap();
		Ok(deserialized_value)
	}
}
