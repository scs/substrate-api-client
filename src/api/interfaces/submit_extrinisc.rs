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

//! Interface to common frame system pallet information.

use crate::{api::ApiResult, Api, Request};
use ac_compose_macros::rpc_params;
use ac_primitives::{ExtrinsicParams, FrameSystemConfig};
use log::*;

/// Simple extrinsic submission without any subscription.
pub trait SubmitExtrinsic {
	type Hash;

	/// Submit an extrsinic to the substrate node, without watching.
	/// Retruns the extrinsic hash.
	fn submit_extrinsic(&self, xthex_prefixed: String) -> ApiResult<Self::Hash>;
}

impl<Signer, Client, Params, Runtime> SubmitExtrinsic for Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	type Hash = Runtime::Hash;

	fn submit_extrinsic(&self, xthex_prefixed: String) -> ApiResult<Self::Hash> {
		debug!("sending extrinsic: {:?}", xthex_prefixed);
		let xt_hash =
			self.client().request("author_submitExtrinsic", rpc_params![xthex_prefixed])?;
		Ok(xt_hash)
	}
}
