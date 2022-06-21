// Copyright 2019-2021 Parity Technologies (UK) Ltd. and Supercomputing Systems AG
// and Integritee AG.
// This file is part of subxt.
//
// subxt is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// subxt is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with subxt.  If not, see <http://www.gnu.org/licenses/>.

//! The errors use in the node-api crate.
//!
//! This file is mostly subxt.

use crate::{
    events::EventsDecodingError,
    metadata::{InvalidMetadataError, Metadata, MetadataError},
};
use derive_more::From;
use sp_core::crypto::SecretStringError;
use sp_runtime::{transaction_validity::TransactionValidityError, DispatchError, ModuleError};

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[cfg(not(feature = "std"))]
use sp_std::vec::Vec;

/// Error enum.
#[derive(Debug, From)]
pub enum Error {
    /// Codec error.
    // #[error("Scale codec error: {0}")]
    // Codec(#[from] codec::Error),
    Codec(codec::Error),
    /// Serde serialization error
    // #[error("Serde json error: {0}")]
    // Serialization(#[from] serde_json::error::Error),
    Serialization(serde_json::error::Error),
    /// Secret string error.
    // #[error("Secret String Error")]
    SecretString(SecretStringError),
    /// Extrinsic validity error
    // #[error("Transaction Validity Error: {0:?}")]
    Invalid(TransactionValidityError),
    /// Invalid metadata error
    // #[error("Invalid Metadata: {0}")]
    InvalidMetadata(InvalidMetadataError),
    /// Invalid metadata error
    // #[error("Metadata: {0}")]
    Metadata(MetadataError),
    /// Runtime error.
    // #[error("Runtime error: {0}")]
    Runtime(RuntimeError),
    /// Events decoding error.
    // #[error("Events decoding error: {0}")]
    EventsDecoding(EventsDecodingError),
    /// Other error.
    // #[error("Other error: {0}")]
    Other(String),
}

/// Runtime error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeError {
    /// Module error.
    // #[error("Runtime module error: {0}")]
    Module(PalletError),
    /// At least one consumer is remaining so the account cannot be destroyed.
    // #[error("At least one consumer is remaining so the account cannot be destroyed.")]
    ConsumerRemaining,
    /// There are too many consumers so the account cannot be created.
    // #[error("There are too many consumers so the account cannot be created.")]
    TooManyConsumers,
    /// There are no providers so the account cannot be created.
    // #[error("There are no providers so the account cannot be created.")]
    NoProviders,
    /// Bad origin.
    // #[error("Bad origin: throw by ensure_signed, ensure_root or ensure_none.")]
    BadOrigin,
    /// Cannot lookup.
    // #[error("Cannot lookup some information required to validate the transaction.")]
    CannotLookup,
    /// Other error.
    // #[error("Other error: {0}")]
    Other(String),
}

impl RuntimeError {
    /// Converts a `DispatchError` into a subxt error.
    pub fn from_dispatch(metadata: &Metadata, error: DispatchError) -> Result<Self, Error> {
        match error {
            DispatchError::Module(ModuleError {
                index,
                error,
                message: _,
            }) => {
                let error = metadata.error(index, error[0])?;
                Ok(Self::Module(PalletError {
                    pallet: error.pallet().to_string(),
                    error: error.error().to_string(),
                    description: error.description().to_vec(),
                }))
            }
            DispatchError::BadOrigin => Ok(Self::BadOrigin),
            DispatchError::CannotLookup => Ok(Self::CannotLookup),
            DispatchError::ConsumerRemaining => Ok(Self::ConsumerRemaining),
            DispatchError::TooManyConsumers => Ok(Self::TooManyConsumers),
            DispatchError::NoProviders => Ok(Self::NoProviders),
            DispatchError::Arithmetic(_math_error) => Ok(Self::Other("math_error".into())),
            DispatchError::Token(_token_error) => Ok(Self::Other("token error".into())),
            DispatchError::Transactional(_transactional_error) => {
                Ok(Self::Other("transactional error".into()))
            }
            DispatchError::Other(msg) => Ok(Self::Other(msg.to_string())),
        }
    }
}

/// Module error.
#[derive(Clone, Debug, Eq, PartialEq)]
// #[error("{error} from {pallet}")]
pub struct PalletError {
    /// The module where the error originated.
    pub pallet: String,
    /// The actual error code.
    pub error: String,
    /// The error description.
    pub description: Vec<String>,
}
