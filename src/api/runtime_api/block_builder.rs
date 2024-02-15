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
use ac_primitives::{config::Config, UncheckedExtrinsicV4};
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use alloc::{vec, vec::Vec};
use sp_core::{Bytes, Encode};
use sp_inherents::{CheckInherentsResult, InherentData};
use sp_runtime::ApplyExtrinsicResult;

#[maybe_async::maybe_async(?Send)]
pub trait BlockBuilderApi: RuntimeApi {
	type ApplyExtrinsicResult;
	type Block;
	type InherentData;
	type CheckInherentsResult;
	type Header;

	/// Apply the given extrinsic.
	async fn apply_extrinsic<Address, Call, Signature, SignedExtra>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
		at_block: Option<Self::Hash>,
	) -> Result<Self::ApplyExtrinsicResult>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode;

	/// Apply the given opaque extrinsic.
	async fn apply_opaque_extrinsic(
		&self,
		extrinsic: Vec<u8>,
		at_block: Option<Self::Hash>,
	) -> Result<Self::ApplyExtrinsicResult>;

	/// Check that the inherents are valid.
	async fn check_inherents(
		&self,
		block: Self::Block,
		data: Self::InherentData,
		at_block: Option<Self::Hash>,
	) -> Result<Self::CheckInherentsResult>;

	/// Finish the current block.
	async fn finalize_block(&self, at_block: Option<Self::Hash>) -> Result<Self::Header>;

	/// Generate inherent extrinsics and return them as encoded Bytes.
	async fn inherent_extrinsics(
		&self,
		inherent: Self::InherentData,
		at_block: Option<Self::Hash>,
	) -> Result<Vec<Bytes>>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> BlockBuilderApi for RuntimeApiClient<T, Client>
where
	T: Config,
	Client: Request,
{
	type ApplyExtrinsicResult = ApplyExtrinsicResult;
	type Block = T::Block;
	type InherentData = InherentData;
	type CheckInherentsResult = CheckInherentsResult;
	type Header = T::Header;

	async fn apply_extrinsic<Address, Call, Signature, SignedExtra>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
		at_block: Option<Self::Hash>,
	) -> Result<Self::ApplyExtrinsicResult>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode,
	{
		self.apply_opaque_extrinsic(extrinsic.encode(), at_block).await
	}

	async fn apply_opaque_extrinsic(
		&self,
		extrinsic: Vec<u8>,
		at_block: Option<Self::Hash>,
	) -> Result<Self::ApplyExtrinsicResult> {
		self.runtime_call("BlockBuilder_apply_extrinsic", vec![extrinsic], at_block)
			.await
	}

	async fn check_inherents(
		&self,
		block: Self::Block,
		data: Self::InherentData,
		at_block: Option<Self::Hash>,
	) -> Result<Self::CheckInherentsResult> {
		self.runtime_call(
			"BlockBuilder_check_inherents",
			vec![block.encode(), data.encode()],
			at_block,
		)
		.await
	}

	async fn finalize_block(&self, at_block: Option<Self::Hash>) -> Result<Self::Header> {
		self.runtime_call("BlockBuilder_finalize_block", vec![], at_block).await
	}

	async fn inherent_extrinsics(
		&self,
		inherent: Self::InherentData,
		at_block: Option<Self::Hash>,
	) -> Result<Vec<Bytes>> {
		self.runtime_call("BlockBuilder_inherent_extrinsics", vec![inherent.encode()], at_block)
			.await
	}
}
