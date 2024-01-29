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
use codec::Encode;
use sp_core::{crypto::KeyTypeId, Bytes};

#[maybe_async::maybe_async(?Send)]
pub trait SessionKeysApi: RuntimeApi {
	type KeyTypeId;

	/// Decode the given public session keys.
	#[allow(clippy::type_complexity)]
	async fn decode_session_keys(
		&self,
		encoded: Bytes,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Vec<(Bytes, Self::KeyTypeId)>>>;

	/// Generate a set of session keys with optionally using the given seed.
	async fn generate_session_keys(
		&self,
		seed: Option<Bytes>,
		at_block: Option<Self::Hash>,
	) -> Result<Bytes>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> SessionKeysApi for RuntimeApiClient<T, Client>
where
	T: Config,
	Client: Request,
{
	type KeyTypeId = KeyTypeId;

	async fn decode_session_keys(
		&self,
		encoded: Bytes,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Vec<(Bytes, Self::KeyTypeId)>>> {
		self.runtime_call("SessionKeys_decode_session_keys", vec![encoded.0], at_block)
			.await
	}

	async fn generate_session_keys(
		&self,
		seed: Option<Bytes>,
		at_block: Option<Self::Hash>,
	) -> Result<Bytes> {
		self.runtime_call("SessionKeys_generate_session_keys", vec![seed.encode()], at_block)
			.await
	}
}
