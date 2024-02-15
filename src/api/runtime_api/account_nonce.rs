/*
   Copyright 2024 Supercomputing Systems AG
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

use super::{RuntimeApi, RuntimeApiClient};
use crate::{api::Result, rpc::Request};
use ac_primitives::config::Config;
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use alloc::vec;
use sp_core::Encode;

#[maybe_async::maybe_async(?Send)]
pub trait AccountNonceApi: RuntimeApi {
	type Index;
	type AccountId;

	/// The API to query account nonce (aka transaction index).
	async fn account_nonce(
		&self,
		account_id: Self::AccountId,
		at_block: Option<Self::Hash>,
	) -> Result<Self::Index>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> AccountNonceApi for RuntimeApiClient<T, Client>
where
	T: Config,
	Client: Request,
{
	type Index = T::Index;
	type AccountId = T::AccountId;

	async fn account_nonce(
		&self,
		account_id: Self::AccountId,
		at_block: Option<Self::Hash>,
	) -> Result<Self::Index> {
		self.runtime_call("AccountNonceApi_account_nonce", vec![account_id.encode()], at_block)
			.await
	}
}
