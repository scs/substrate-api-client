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

pub use crate::{
	std::error::{ApiResult, Error as ApiClientError},
	utils::FromHexString,
};
pub use api::Api;
pub use frame_metadata::RuntimeMetadataPrefixed;
pub use pallet_transaction_payment::FeeDetails;
pub use serde_json::Value;

pub use sp_core::{crypto::Pair, storage::StorageKey};
pub use sp_runtime::{
	generic::SignedBlock,
	traits::{Block, Header, IdentifyAccount},
	AccountId32 as AccountId, MultiSignature, MultiSigner,
};
pub use sp_std::prelude::*;
pub use sp_version::RuntimeVersion;

use serde::{Deserialize, Serialize};
use sp_core::H256 as Hash;

pub mod api;
pub mod error;
pub mod rpc;

use crate::rpc::json_req;

pub trait RpcClient {
	/// Sends a RPC request that returns a String
	fn get_request(&self, jsonreq: serde_json::Value) -> ApiResult<String>;

	/// Send a RPC request that returns a SHA256 hash
	fn send_extrinsic(&self, xthex_prefixed: String, exit_on: XtStatus) -> ApiResult<Option<Hash>>;
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum XtStatus {
	Unknown = 0,
	/// uses `author_submit` without watching.
	SubmitOnly = 1,
	Ready = 2,
	Broadcast = 3,
	InBlock = 4,
	Finalized = 5,
	Future = 10,
}

// Exact structure from
// https://github.com/paritytech/substrate/blob/master/client/rpc-api/src/state/helpers.rs
// Adding manually so we don't need sc-rpc-api, which brings in async dependencies
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadProof<Hash> {
	/// Block hash used to generate the proof
	pub at: Hash,
	/// A proof used to prove that storage entries are included in the storage trie
	pub proof: Vec<sp_core::Bytes>,
}
