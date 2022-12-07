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

//! Extrinsics for `pallet-utility`.

use super::common::Batch;
use crate::{rpc::RpcClient, Api};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{CallIndex, ExtrinsicParams, FrameSystemConfig, UncheckedExtrinsicV4};
use codec::Encode;
use sp_core::Pair;
use sp_runtime::MultiSignature;

const UTILITY_MODULE: &str = "Utility";
const UTILITY_BATCH: &str = "batch";
const UTILITY_FORCE_BATCH: &str = "force_batch";

pub type UtilityBatchFn<Call> = (CallIndex, Batch<Call>);
pub type UtilityBatchXt<Call, SignedExtra> =
	UncheckedExtrinsicV4<UtilityBatchFn<Call>, SignedExtra>;

impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Signer: Pair,
	MultiSignature: From<Signer::Signature>,
	Client: RpcClient,
	Runtime: FrameSystemConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime::AccountId: From<Signer::Public>,
{
	pub fn batch<Call: Encode + Clone>(
		&self,
		calls: Vec<Call>,
	) -> UtilityBatchXt<Call, Params::SignedExtra> {
		let calls = Batch { calls };
		compose_extrinsic!(self, UTILITY_MODULE, UTILITY_BATCH, calls)
	}

	pub fn force_batch<Call: Encode + Clone>(
		&self,
		calls: Vec<Call>,
	) -> UtilityBatchXt<Call, Params::SignedExtra> {
		let calls = Batch { calls };
		compose_extrinsic!(self, UTILITY_MODULE, UTILITY_FORCE_BATCH, calls)
	}
}
