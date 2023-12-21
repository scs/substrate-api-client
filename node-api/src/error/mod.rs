// This file was taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! General node-api Error implementation.

use alloc::{boxed::Box, string::String, vec::Vec};
use core::fmt::Debug;
use derive_more::From;

// Re-expose the errors we use from other crates here:
pub use crate::metadata::{MetadataConversionError, MetadataError};
pub use scale_decode::Error as DecodeError;
pub use scale_encode::Error as EncodeError;
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
	/// Invalid metadata error
	InvalidMetadata(MetadataConversionError),
	/// Invalid metadata error
	Metadata(MetadataError),
	/// Runtime error.
	Runtime(DispatchError),
	/// Error decoding to a [`crate::dynamic::Value`].
	DecodeValue(Box<DecodeError>),
	/// Error encoding from a [`crate::dynamic::Value`].
	EncodeValue(Box<EncodeError>),
	/// The bytes representing an error that we were unable to decode.
	Unknown(Vec<u8>),
	/// Other error.
	Other(String),
}

impl From<&str> for Error {
	fn from(error: &str) -> Self {
		Error::Other(error.into())
	}
}

impl From<DecodeError> for Error {
	fn from(error: DecodeError) -> Self {
		Error::DecodeValue(Box::new(error))
	}
}

impl From<EncodeError> for Error {
	fn from(error: EncodeError) -> Self {
		Error::EncodeValue(Box::new(error))
	}
}
