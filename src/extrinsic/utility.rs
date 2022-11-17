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
use crate::{Api, RpcClient};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{CallIndex, ExtrinsicParams, UncheckedExtrinsicV4};
use codec::Encode;
use sp_core::Pair;
use sp_runtime::{MultiSignature, MultiSigner};

const UTILITY_MODULE: &str = "Utility";
const UTILITY_BATCH: &str = "batch";
const UTILITY_FORCE_BATCH: &str = "force_batch";

pub type UtilityBatchFn<Call> = (CallIndex, Batch<Call>);
pub type UtilityBatchXt<Call, SignedExtra> =
	UncheckedExtrinsicV4<UtilityBatchFn<Call>, SignedExtra>;

impl<P, Client, Params> Api<P, Client, Params>
where
	P: Pair,
	MultiSignature: From<P::Signature>,
	MultiSigner: From<P::Public>,
	Client: RpcClient,
	Params: ExtrinsicParams,
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
