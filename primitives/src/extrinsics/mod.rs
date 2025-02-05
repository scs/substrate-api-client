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

//! Primitives for substrate extrinsics.

pub use extrinsic_params::{
	AssetTip, ExtrinsicParams, GenericAdditionalParams, GenericExtrinsicParams, GenericImplicit,
	GenericTxExtension, PlainTip, SignedPayload,
};
#[allow(deprecated)]
pub use extrinsic_v4::deprecated;
pub use signer::{ExtrinsicSigner, SignExtrinsic};
pub use sp_runtime::generic::{Preamble, UncheckedExtrinsic};

/// Call Index used a prefix of every extrinsic call.
pub type CallIndex = [u8; 2];

pub mod extrinsic_params;
pub mod extrinsic_params_without_hash_check;
mod extrinsic_v4;
pub mod signer;
