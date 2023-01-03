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

use ac_node_api::EventDetails;
use ac_primitives::Bytes;
use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

/// Extrinsic report returned upon a submit_and_watch request.
/// Holds as much information as available.
#[derive(Debug, Clone)]
pub struct ExtrinsicReport<Hash> {
	// Hash of the extrinsic.
	pub extrinsic_hash: Hash,
	// Block hash of the block the extrinsic was included in.
	// Only available if watched until at least `InBlock`.
	pub block_hash: Option<Hash>,
	// Last known Transaction Status.
	pub status: TransactionStatus<Hash, Hash>,
	// Events assosciated to the extrinsic.
	// Only available if explicitly stated, because
	// extra node queries are necessary to fetch the events.
	pub events: Option<Vec<EventDetails>>,
}

impl<Hash> ExtrinsicReport<Hash> {
	pub fn new(
		extrinsic_hash: Hash,
		block_hash: Option<Hash>,
		status: TransactionStatus<Hash, Hash>,
		events: Option<Vec<EventDetails>>,
	) -> Self {
		Self { extrinsic_hash, block_hash, status, events }
	}
}

/// Simplified TransactionStatus to allow the user to choose until when to watch
/// an extrinsic.
// Indexes must match the TransactionStatus::as_u8 from below.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum XtStatus {
	Ready = 1,
	Broadcast = 2,
	InBlock = 3,
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

	/// Returns true if the input status has been reached (or overreached)
	/// and false in case the status is not yet on the expected level.
	pub fn reached_status(&self, status: XtStatus) -> bool {
		self.as_u8() >= status as u8
	}

	pub fn get_maybe_block_hash(&self) -> Option<&BlockHash> {
		match self {
			TransactionStatus::InBlock(block_hash) => Some(block_hash),
			TransactionStatus::Retracted(block_hash) => Some(block_hash),
			TransactionStatus::FinalityTimeout(block_hash) => Some(block_hash),
			TransactionStatus::Finalized(block_hash) => Some(block_hash),
			_ => None,
		}
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
	pub proof: Vec<Bytes>,
}

#[cfg(test)]
mod tests {
	use super::{TransactionStatus as GenericTransactionStatus, *};
	use sp_core::H256;

	type TransactionStatus = GenericTransactionStatus<H256, H256>;

	#[test]
	fn test_xt_status_as_u8() {
		assert_eq!(1, XtStatus::Ready as u8);
		assert_eq!(2, XtStatus::Broadcast as u8);
		assert_eq!(3, XtStatus::InBlock as u8);
		assert_eq!(6, XtStatus::Finalized as u8);
	}

	#[test]
	fn test_transaction_status_as_u8() {
		assert_eq!(0, TransactionStatus::Future.as_u8());
		assert_eq!(1, TransactionStatus::Ready.as_u8());
		assert_eq!(2, TransactionStatus::Broadcast(vec![]).as_u8());
		assert_eq!(3, TransactionStatus::InBlock(H256::random()).as_u8());
		assert_eq!(4, TransactionStatus::Retracted(H256::random()).as_u8());
		assert_eq!(5, TransactionStatus::FinalityTimeout(H256::random()).as_u8());
		assert_eq!(6, TransactionStatus::Finalized(H256::random()).as_u8());
		assert_eq!(7, TransactionStatus::Usurped(H256::random()).as_u8());
		assert_eq!(8, TransactionStatus::Dropped.as_u8());
		assert_eq!(9, TransactionStatus::Invalid.as_u8());
	}

	#[test]
	fn test_transaction_status_is_supported() {
		// Supported.
		assert!(TransactionStatus::Ready.is_supported());
		assert!(TransactionStatus::Broadcast(vec![]).is_supported());
		assert!(TransactionStatus::InBlock(H256::random()).is_supported());
		assert!(TransactionStatus::FinalityTimeout(H256::random()).is_supported());
		assert!(TransactionStatus::Finalized(H256::random()).is_supported());

		// Not supported.
		assert!(!TransactionStatus::Future.is_supported());
		assert!(!TransactionStatus::Retracted(H256::random()).is_supported());
		assert!(!TransactionStatus::Usurped(H256::random()).is_supported());
		assert!(!TransactionStatus::Dropped.is_supported());
		assert!(!TransactionStatus::Invalid.is_supported());
	}

	#[test]
	fn test_reached_xt_status_for_ready() {
		let status = XtStatus::Ready;

		// Has not yet reached XtStatus.
		assert!(!TransactionStatus::Future.reached_status(status));

		// Reached XtStatus.
		assert!(TransactionStatus::Ready.reached_status(status));
		assert!(TransactionStatus::Broadcast(vec![]).reached_status(status));
		assert!(TransactionStatus::InBlock(H256::random()).reached_status(status));
		assert!(TransactionStatus::FinalityTimeout(H256::random()).reached_status(status));
		assert!(TransactionStatus::Finalized(H256::random()).reached_status(status));
		assert!(TransactionStatus::Retracted(H256::random()).reached_status(status));
		assert!(TransactionStatus::Usurped(H256::random()).reached_status(status));
		assert!(TransactionStatus::Dropped.reached_status(status));
		assert!(TransactionStatus::Invalid.reached_status(status));
	}

	#[test]
	fn test_reached_xt_status_for_broadcast() {
		let status = XtStatus::Broadcast;

		// Has not yet reached XtStatus.
		assert!(!TransactionStatus::Future.reached_status(status));
		assert!(!TransactionStatus::Ready.reached_status(status));

		// Reached XtStatus.
		assert!(TransactionStatus::Broadcast(vec![]).reached_status(status));
		assert!(TransactionStatus::InBlock(H256::random()).reached_status(status));
		assert!(TransactionStatus::FinalityTimeout(H256::random()).reached_status(status));
		assert!(TransactionStatus::Finalized(H256::random()).reached_status(status));
		assert!(TransactionStatus::Retracted(H256::random()).reached_status(status));
		assert!(TransactionStatus::Usurped(H256::random()).reached_status(status));
		assert!(TransactionStatus::Dropped.reached_status(status));
		assert!(TransactionStatus::Invalid.reached_status(status));
	}

	#[test]
	fn test_reached_xt_status_for_in_block() {
		let status = XtStatus::InBlock;

		// Has not yet reached XtStatus.
		assert!(!TransactionStatus::Future.reached_status(status));
		assert!(!TransactionStatus::Ready.reached_status(status));
		assert!(!TransactionStatus::Broadcast(vec![]).reached_status(status));

		// Reached XtStatus.
		assert!(TransactionStatus::InBlock(H256::random()).reached_status(status));
		assert!(TransactionStatus::FinalityTimeout(H256::random()).reached_status(status));
		assert!(TransactionStatus::Finalized(H256::random()).reached_status(status));
		assert!(TransactionStatus::Retracted(H256::random()).reached_status(status));
		assert!(TransactionStatus::Usurped(H256::random()).reached_status(status));
		assert!(TransactionStatus::Dropped.reached_status(status));
		assert!(TransactionStatus::Invalid.reached_status(status));
	}

	#[test]
	fn test_reached_xt_status_for_finalized() {
		let status = XtStatus::Finalized;

		// Has not yet reached XtStatus.
		assert!(!TransactionStatus::Future.reached_status(status));
		assert!(!TransactionStatus::Ready.reached_status(status));
		assert!(!TransactionStatus::Broadcast(vec![]).reached_status(status));
		assert!(!TransactionStatus::InBlock(H256::random()).reached_status(status));
		assert!(!TransactionStatus::Retracted(H256::random()).reached_status(status));
		assert!(!TransactionStatus::FinalityTimeout(H256::random()).reached_status(status));

		// Reached XtStatus.
		assert!(TransactionStatus::Finalized(H256::random()).reached_status(status));
		assert!(TransactionStatus::Usurped(H256::random()).reached_status(status));
		assert!(TransactionStatus::Dropped.reached_status(status));
		assert!(TransactionStatus::Invalid.reached_status(status));
	}
}
