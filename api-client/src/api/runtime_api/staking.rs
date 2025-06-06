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
#[cfg(all(not(feature = "sync-api"), not(feature = "std")))]
use alloc::boxed::Box;
use alloc::vec;
use sp_core::Encode;

#[maybe_async::maybe_async(?Send)]
pub trait StakingApi: RuntimeApi {
	type Balance;

	/// Returns the nominations quota for a nominator with a given balance.
	async fn nominations_quota(
		&self,
		balance: Self::Balance,
		at_block: Option<Self::Hash>,
	) -> Result<u32>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> StakingApi for RuntimeApiClient<T, Client>
where
	T: Config,
	Client: Request,
{
	type Balance = T::Balance;

	async fn nominations_quota(
		&self,
		balance: Self::Balance,
		at_block: Option<Self::Hash>,
	) -> Result<u32> {
		self.runtime_call("StakingApi_nominations_quota", vec![balance.encode()], at_block)
			.await
	}
}
