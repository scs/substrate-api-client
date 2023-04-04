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
use ac_primitives::{
	config::Config, extrinsic_params::ExtrinsicParams, SignExtrinsic, UncheckedExtrinsicV4,
};
use codec::Encode;

impl<T: Config, Signer, Client, Block> Api<T, Signer, Client, Block>
where
	Signer: SignExtrinsic<T::AccountId>,
{
	/// Wrapper around the `compose_extrinsic_offline!` macro to be less verbose.
	pub fn compose_extrinsic_offline<Call: Encode + Clone>(
		&self,
		call: Call,
		nonce: T::Index,
	) -> UncheckedExtrinsicV4<
		Signer::ExtrinsicAddress,
		Call,
		Signer::Signature,
		<T::ExtrinsicParams as ExtrinsicParams<T::Index, T::Hash>>::SignedExtra,
	> {
		match self.signer() {
			Some(signer) => compose_extrinsic_offline!(signer, call, self.extrinsic_params(nonce)),
			None => UncheckedExtrinsicV4 { signature: None, function: call },
		}
	}
}
