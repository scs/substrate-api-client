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

pub use self::{
	account_nonce::*, api_core::*, authority_discovery::*, block_builder::*, metadata::*, mmr::*,
	session_keys::*, staking::*, transaction_payment::*, transaction_payment_call::*,
};

pub mod account_nonce;
pub mod api_core;
pub mod authority_discovery;
pub mod block_builder;
pub mod metadata;
pub mod mmr;
pub mod session_keys;
pub mod staking;
pub mod transaction_payment;
pub mod transaction_payment_call;

use crate::{api::Result, rpc::Request};
use ac_compose_macros::rpc_params;
use ac_primitives::config::Config;
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use alloc::{sync::Arc, vec::Vec};
use codec::Decode;
use core::marker::PhantomData;
use sp_core::Bytes;

#[derive(Clone)]
pub struct RuntimeApiClient<T, Client> {
	client: Arc<Client>,
	_phantom: PhantomData<T>,
}

impl<T, Client> RuntimeApiClient<T, Client> {
	pub fn new(client: Arc<Client>) -> Self {
		Self { client, _phantom: PhantomData }
	}
}

#[maybe_async::maybe_async(?Send)]
pub trait RuntimeApi {
	type Hash;

	/// Query a runtime api call with automatic decoding to the expected return type.
	async fn runtime_call<V: Decode>(
		&self,
		method: &str,
		data: Vec<Vec<u8>>,
		at_block: Option<Self::Hash>,
	) -> Result<V>;

	/// Query a raw runtime api call without decoding.
	async fn opaque_runtime_call(
		&self,
		method: &str,
		data: Vec<Vec<u8>>,
		at_block: Option<Self::Hash>,
	) -> Result<Bytes>;

	// Perform a rpc call to a builtin on the chain.
	async fn rpc_call(
		&self,
		method: &str,
		data: Option<Bytes>,
		at_block: Option<Self::Hash>,
	) -> Result<Bytes>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> RuntimeApi for RuntimeApiClient<T, Client>
where
	T: Config,
	Client: Request,
{
	type Hash = T::Hash;

	async fn runtime_call<V: Decode>(
		&self,
		method: &str,
		data: Vec<Vec<u8>>,
		at_block: Option<Self::Hash>,
	) -> Result<V> {
		let bytes = self.opaque_runtime_call(method, data, at_block).await?;
		Ok(Decode::decode(&mut bytes.0.as_slice())?)
	}

	async fn opaque_runtime_call(
		&self,
		method: &str,
		data: Vec<Vec<u8>>,
		at_block: Option<Self::Hash>,
	) -> Result<Bytes> {
		let data = match data.is_empty() {
			true => None,
			false => {
				let mut appended_data = Vec::new();
				for mut item in data {
					appended_data.append(&mut item);
				}
				Some(appended_data.into())
			},
		};
		self.rpc_call(method, data, at_block).await
	}

	async fn rpc_call(
		&self,
		method: &str,
		data: Option<Bytes>,
		at_block: Option<Self::Hash>,
	) -> Result<Bytes> {
		let extracted_data: Bytes = match data {
			Some(data) => data,
			None => Vec::new().into(),
		};
		let return_bytes = self
			.client
			.request("state_call", rpc_params![method, extracted_data, at_block])
			.await?;
		Ok(return_bytes)
	}
}
