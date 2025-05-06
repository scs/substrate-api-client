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
use ac_primitives::{config::Config, RuntimeVersion};
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use alloc::vec;
use sp_core::{Bytes, Encode};

#[maybe_async::maybe_async(?Send)]
pub trait CoreApi: RuntimeApi {
	type Block;
	type Header;
	type RuntimeVersion;

	/// Execute the given block.
	async fn execute_block(&self, block: Self::Block, at_block: Option<Self::Hash>) -> Result<()>;

	/// Execute the given opaque block.
	async fn execute_opaque_block(&self, block: Bytes, at_block: Option<Self::Hash>) -> Result<()>;

	/// Initialize a block with the given header.
	async fn initialize_block(
		&self,
		header: Self::Header,
		at_block: Option<Self::Hash>,
	) -> Result<()>;

	/// Returns the version of the runtime.
	async fn version(&self, at_block: Option<Self::Hash>) -> Result<Self::RuntimeVersion>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> CoreApi for RuntimeApiClient<T, Client>
where
	T: Config,
	Client: Request,
{
	type Block = T::Block;
	type Header = T::Header;
	type RuntimeVersion = RuntimeVersion;

	async fn execute_block(&self, block: Self::Block, at_block: Option<Self::Hash>) -> Result<()> {
		self.execute_opaque_block(block.encode().into(), at_block).await
	}

	async fn execute_opaque_block(&self, block: Bytes, at_block: Option<Self::Hash>) -> Result<()> {
		self.runtime_call("Core_execute_block", vec![block.0], at_block).await
	}

	async fn initialize_block(
		&self,
		header: Self::Header,
		at_block: Option<Self::Hash>,
	) -> Result<()> {
		self.runtime_call("Core_initialize_block", vec![header.encode()], at_block)
			.await
	}

	async fn version(&self, at_block: Option<Self::Hash>) -> Result<Self::RuntimeVersion> {
		self.runtime_call("Core_version", vec![], at_block).await
	}
}
