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

//! Helper function to easily create extrinsics offline (without getter calls to the node).

use crate::Api;
use ac_compose_macros::compose_extrinsic_offline;
use ac_primitives::{ExtrinsicParams, FrameSystemConfig, UncheckedExtrinsicV4};
use codec::Encode;
use sp_core::Pair;
use sp_runtime::{MultiSignature, MultiSigner};

impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Signer: Pair,
	MultiSignature: From<Signer::Signature>,
	MultiSigner: From<Signer::Public>,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
	Runtime::AccountId: From<Signer::Public>,
{
	/// Wrapper around the `compose_extrinsic_offline!` macro to be less verbose.
	pub fn compose_extrinsic_offline<Call: Encode + Clone>(
		&self,
		call: Call,
		nonce: Runtime::Index,
	) -> UncheckedExtrinsicV4<Call, Params::SignedExtra> {
		match self.signer() {
			Some(signer) => compose_extrinsic_offline!(signer, call, self.extrinsic_params(nonce)),
			None => UncheckedExtrinsicV4 { signature: None, function: call },
		}
	}
}
