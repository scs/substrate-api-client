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
use ac_primitives::{config::Config, EncodableOpaqueLeaf, MmrError, Proof};
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use alloc::{vec, vec::Vec};
use core::result::Result as StdResult;
use sp_core::Encode;

#[maybe_async::maybe_async(?Send)]
pub trait MmrApi: RuntimeApi {
	type Error;
	type BlockNumber;
	type EncodableOpaqueLeaf;
	type Proof;

	/// Generate MMR proof for the given block numbers.
	#[allow(clippy::type_complexity)]
	async fn generate_proof(
		&self,
		block_numbers: Vec<Self::BlockNumber>,
		best_known_block_number: Option<Self::BlockNumber>,
		at_block: Option<Self::Hash>,
	) -> Result<StdResult<(Vec<Self::EncodableOpaqueLeaf>, Self::Proof), Self::Error>>;

	/// Return the on-chain MMR root hash.
	async fn root(
		&self,
		at_block: Option<Self::Hash>,
	) -> Result<StdResult<Vec<Self::Hash>, Self::Error>>;

	/// Verify MMR proof against on-chain MMR.
	async fn verify_proof(
		&self,
		leaves: Vec<Self::EncodableOpaqueLeaf>,
		proof: Self::Proof,
		at_block: Option<Self::Hash>,
	) -> Result<StdResult<(), Self::Error>>;

	/// Verify MMR proof against given root hash.
	async fn verify_proof_stateless(
		&self,
		root: Self::Hash,
		leaves: Vec<Self::EncodableOpaqueLeaf>,
		proof: Self::Proof,
		at_block: Option<Self::Hash>,
	) -> Result<StdResult<(), Self::Error>>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> MmrApi for RuntimeApiClient<T, Client>
where
	T: Config,
	Client: Request,
{
	type Error = MmrError;
	type BlockNumber = T::BlockNumber;
	type EncodableOpaqueLeaf = EncodableOpaqueLeaf;
	type Proof = Proof<T::Hash>;

	async fn generate_proof(
		&self,
		block_numbers: Vec<Self::BlockNumber>,
		best_known_block_number: Option<Self::BlockNumber>,
		at_block: Option<Self::Hash>,
	) -> Result<StdResult<(Vec<Self::EncodableOpaqueLeaf>, Self::Proof), Self::Error>> {
		self.runtime_call(
			"MmrApi_generate_proof",
			vec![block_numbers.encode(), best_known_block_number.encode()],
			at_block,
		)
		.await
	}

	async fn root(
		&self,
		at_block: Option<Self::Hash>,
	) -> Result<StdResult<Vec<Self::Hash>, Self::Error>> {
		self.runtime_call("MmrApi_root", vec![], at_block).await
	}

	async fn verify_proof(
		&self,
		leaves: Vec<Self::EncodableOpaqueLeaf>,
		proof: Self::Proof,
		at_block: Option<Self::Hash>,
	) -> Result<StdResult<(), Self::Error>> {
		self.runtime_call("MmrApi_verify_proof", vec![leaves.encode(), proof.encode()], at_block)
			.await
	}

	async fn verify_proof_stateless(
		&self,
		root: Self::Hash,
		leaves: Vec<Self::EncodableOpaqueLeaf>,
		proof: Self::Proof,
		at_block: Option<Self::Hash>,
	) -> Result<StdResult<(), Self::Error>> {
		self.runtime_call(
			"MmrApi_verify_proof_stateless",
			vec![root.encode(), leaves.encode(), proof.encode()],
			at_block,
		)
		.await
	}
}
