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

pub use api_client::*;
pub use error::{Error, Result};
pub use rpc_api::*;

pub mod api_client;
pub mod error;
pub mod rpc_api;

use serde::{Deserialize, Serialize};

/// Simplified TransactionStatus to allow the user to choose until when to watch
/// an extrinsic.
// Indexes must match the TransactionStatus::as_u8 from below.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum XtStatus {
	Ready = 1,
	Broadcast = 2,
	InBlock = 4,
	Finalized = 6,
}

/// Possible transaction status events.
// Copied from `sc-transaction-pool`
// (https://github.com/paritytech/substrate/blob/dddfed3d9260cf03244f15ba3db4edf9af7467e9/client/transaction-pool/api/src/lib.rs)
// as the library is not no-std compatible
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionStatus<Hash, BlockHash> {
	/// Transaction is part of the future queue.
	Future,
	/// Transaction is part of the ready queue.
	Ready,
	/// The transaction has been broadcast to the given peers.
	Broadcast(Vec<String>),
	/// Transaction has been included in block with given hash.
	InBlock(BlockHash),
	/// The block this transaction was included in has been retracted.
	Retracted(BlockHash),
	/// Maximum number of finality watchers has been reached,
	/// old watchers are being removed.
	FinalityTimeout(BlockHash),
	/// Transaction has been finalized by a finality-gadget, e.g GRANDPA
	Finalized(BlockHash),
	/// Transaction has been replaced in the pool, by another transaction
	/// that provides the same tags. (e.g. same (sender, nonce)).
	Usurped(Hash),
	/// Transaction has been dropped from the pool because of the limit.
	Dropped,
	/// Transaction is no longer valid in the current state.
	Invalid,
}

impl<Hash, BlockHash> TransactionStatus<Hash, BlockHash> {
	pub fn as_u8(&self) -> u8 {
		match self {
			TransactionStatus::Future => 0,
			TransactionStatus::Ready => 1,
			TransactionStatus::Broadcast(_) => 2,
			TransactionStatus::InBlock(_) => 3,
			TransactionStatus::Retracted(_) => 4,
			TransactionStatus::FinalityTimeout(_) => 5,
			TransactionStatus::Finalized(_) => 6,
			TransactionStatus::Usurped(_) => 7,
			TransactionStatus::Dropped => 8,
			TransactionStatus::Invalid => 9,
		}
	}

	pub fn is_supported(&self) -> bool {
		matches!(
			self,
			TransactionStatus::Ready
				| TransactionStatus::Broadcast(_)
				| TransactionStatus::InBlock(_)
				| TransactionStatus::FinalityTimeout(_)
				| TransactionStatus::Finalized(_)
		)
	}
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
