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
use alloc::{vec, vec::Vec};
use codec::Decode;

#[maybe_async::maybe_async(?Send)]
pub trait AuthorityDiscoveryApi: RuntimeApi {
	/// Retrieve authority identifiers of the current and next authority set.
	async fn authorities<AuthorityId: Decode>(
		&self,
		at_block: Option<Self::Hash>,
	) -> Result<Vec<AuthorityId>>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> AuthorityDiscoveryApi for RuntimeApiClient<T, Client>
where
	T: Config,
	Client: Request,
{
	async fn authorities<AuthorityId: Decode>(
		&self,
		at_block: Option<Self::Hash>,
	) -> Result<Vec<AuthorityId>> {
		self.runtime_call("AuthorityDiscoveryApi_authorities", vec![], at_block).await
	}
}
