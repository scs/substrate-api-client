// This file was taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! General node-api Error implementation.

use alloc::{format, string::String};
use core::fmt::Debug;
use derive_more::From;

// Re-expose the errors we use from other crates here:
pub use crate::{
	metadata::{InvalidMetadataError, MetadataError},
	scale_value::{DecodeError, EncodeError},
};
pub use sp_core::crypto::SecretStringError;
pub use sp_runtime::transaction_validity::TransactionValidityError;

mod dispatch_error;
pub use dispatch_error::*;

/// The underlying error enum, generic over the type held by the `Runtime`
/// variant. Prefer to use the [`Error<E>`] and [`Error`] aliases over
/// using this type directly.
#[derive(Debug, From)]
pub enum Error {
	/// Codec error.
	Codec(codec::Error),
	/// Serde serialization error
	Serialization(serde_json::error::Error),
	/// Secret string error.
	SecretString(SecretStringError),
	/// Extrinsic validity error
	Invalid(TransactionValidityError),
	/// Invalid metadata error
	InvalidMetadata(InvalidMetadataError),
	/// Invalid metadata error
	Metadata(MetadataError),
	/// Runtime error.
	Runtime(DispatchError),
	/// Error decoding to a [`crate::dynamic::Value`].
	DecodeValue(DecodeError),
	/// Error encoding from a [`crate::dynamic::Value`].
	EncodeValue(EncodeError),
	/// Transaction progress error.
	Transaction(TransactionError),
	/// Block related error.
	Block(BlockError),
	/// An error encoding a storage address.
	StorageAddress(StorageAddressError),
	/// Other error.
	Other(String),
}

impl From<&str> for Error {
	fn from(error: &str) -> Self {
		Error::Other(error.into())
	}
}

/// Block error
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlockError {
	/// The block
	BlockHashNotFound(String),
}

impl BlockError {
	/// Produce an error that a block with the given hash cannot be found.
	pub fn block_hash_not_found(hash: impl AsRef<[u8]>) -> BlockError {
		let hash = format!("0x{}", hex::encode(hash));
		BlockError::BlockHashNotFound(hash)
	}
}

/// Transaction error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionError {
	/// The finality subscription expired (after ~512 blocks we give up if the
	/// block hasn't yet been finalized).
	FinalitySubscriptionTimeout,
	/// The block hash that the transaction was added to could not be found.
	/// This is probably because the block was retracted before being finalized.
	BlockHashNotFound,
}

/// Something went wrong trying to encode a storage address.
#[derive(Clone, Debug)]
pub enum StorageAddressError {
	/// Storage map type must be a composite type.
	MapTypeMustBeTuple,
	/// Storage lookup does not have the expected number of keys.
	WrongNumberOfKeys {
		/// The actual number of keys needed, based on the metadata.
		actual: usize,
		/// The number of keys provided in the storage address.
		expected: usize,
	},
	/// Storage lookup requires a type that wasn't found in the metadata.
	TypeNotFound(u32),
	/// This storage entry in the metadata does not have the correct number of hashers to fields.
	WrongNumberOfHashers {
		/// The number of hashers in the metadata for this storage entry.
		hashers: usize,
		/// The number of fields in the metadata for this storage entry.
		fields: usize,
	},
}
