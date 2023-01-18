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
//! https://polkadot.js.org/docs/substrate/extrinsics/#utility

use crate::{rpc::Request, Api};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{
	CallIndex, ExtrinsicParams, FrameSystemConfig, SignExtrinsic, UncheckedExtrinsicV4,
};
use alloc::{borrow::ToOwned, vec::Vec};
use codec::{Decode, Encode};

const UTILITY_MODULE: &str = "Utility";
const BATCH: &str = "batch";
const FORCE_BATCH: &str = "force_batch";

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug)]
pub struct Batch<Call> {
	pub calls: Vec<Call>,
}

pub type BatchCall<Call> = (CallIndex, Batch<Call>);

pub trait UtilityExtrinsics {
	type Extrinsic<Call>;

	// Send a batch of dispatch calls.
	fn batch<Call: Encode + Clone>(&self, calls: Vec<Call>) -> Self::Extrinsic<BatchCall<Call>>;

	// Send a batch of dispatch calls. Unlike batch, it allows errors and won't interrupt.
	fn force_batch<Call: Encode + Clone>(
		&self,
		calls: Vec<Call>,
	) -> Self::Extrinsic<BatchCall<Call>>;
}

impl<Signer, Client, Params, Runtime> UtilityExtrinsics for Api<Signer, Client, Params, Runtime>
where
	Signer: SignExtrinsic<Runtime::AccountId>,
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	type Extrinsic<Call> = UncheckedExtrinsicV4<
		Signer::ExtrinsicAddress,
		Call,
		Signer::Signature,
		Params::SignedExtra,
	>;

	fn batch<Call: Encode + Clone>(&self, calls: Vec<Call>) -> Self::Extrinsic<BatchCall<Call>> {
		let calls = Batch { calls };
		compose_extrinsic!(self, UTILITY_MODULE, BATCH, calls)
	}

	fn force_batch<Call: Encode + Clone>(
		&self,
		calls: Vec<Call>,
	) -> Self::Extrinsic<BatchCall<Call>> {
		let calls = Batch { calls };
		compose_extrinsic!(self, UTILITY_MODULE, FORCE_BATCH, calls)
	}
}
