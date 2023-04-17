// This file was taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! The errors use in the node-api crate.
//!
//! This file is mostly subxt.

use crate::{
	alloc::{
		borrow::Cow,
		format,
		string::{String, ToString},
		vec::Vec,
	},
	metadata::Metadata,
};

use codec::Decode;
use core::fmt::Debug;
use derive_more::From;
use log::*;
use scale_info::TypeDef;

// Re-expose the errors we use from other crates here:
pub use crate::{
	decoder::{DecodeError, EncodeError},
	metadata::{InvalidMetadataError, MetadataError},
};
pub use sp_core::crypto::SecretStringError;
pub use sp_runtime::transaction_validity::TransactionValidityError;

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
	EncodeValue(EncodeError<()>),
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
/// This is our attempt to decode a runtime DispatchError. We either
/// successfully decode it into a [`ModuleError`], or we fail and keep
/// hold of the bytes, which we can attempt to decode if we have an
/// appropriate static type to hand.
#[derive(Debug)]
pub enum DispatchError {
	/// An error was emitted from a specific pallet/module.
	Module(ModuleError),
	/// Some other error was emitted.
	Other(Vec<u8>),
}

impl DispatchError {
	/// Attempt to decode a runtime DispatchError, returning either the [`ModuleError`] it decodes
	/// to, along with additional details on the error, or returning the raw bytes if it could not
	/// be decoded.
	pub fn decode_from<'a>(bytes: impl Into<Cow<'a, [u8]>>, metadata: &Metadata) -> Self {
		let bytes = bytes.into();

		let dispatch_error_ty_id = match metadata.dispatch_error_ty() {
			Some(id) => id,
			None => {
				warn!("Can't decode error: sp_runtime::DispatchError was not found in Metadata");
				return DispatchError::Other(bytes.into_owned())
			},
		};

		let dispatch_error_ty = match metadata.types().resolve(dispatch_error_ty_id) {
			Some(ty) => ty,
			None => {
				warn!("Can't decode error: sp_runtime::DispatchError type ID doesn't resolve to a known type");
				return DispatchError::Other(bytes.into_owned())
			},
		};

		let variant = match &dispatch_error_ty.type_def {
			TypeDef::Variant(var) => var,
			_ => {
				warn!("Can't decode error: sp_runtime::DispatchError type is not a Variant");
				return DispatchError::Other(bytes.into_owned())
			},
		};

		let module_variant_idx =
			variant.variants.iter().find(|v| v.name == "Module").map(|v| v.index);
		let module_variant_idx = match module_variant_idx {
			Some(idx) => idx,
			None => {
				warn!("Can't decode error: sp_runtime::DispatchError does not have a 'Module' variant");
				return DispatchError::Other(bytes.into_owned())
			},
		};

		// If the error bytes don't correspond to a ModuleError, just return the bytes.
		// This is perfectly reasonable and expected, so no logging.
		if bytes[0] != module_variant_idx {
			return DispatchError::Other(bytes.into_owned())
		}

		// The remaining bytes are the module error, all being well:
		let bytes = &bytes[1..];

		// The oldest and second oldest type of error decode to this shape:
		#[derive(Decode)]
		struct LegacyModuleError {
			index: u8,
			error: u8,
		}

		// The newer case expands the error for forward compat:
		#[derive(Decode)]
		struct CurrentModuleError {
			index: u8,
			error: [u8; 4],
		}

		// try to decode into the new shape, or the old if that doesn't work
		let err = match CurrentModuleError::decode(&mut &*bytes) {
			Ok(e) => e,
			Err(_) => {
				let old_e = match LegacyModuleError::decode(&mut &*bytes) {
					Ok(err) => err,
					Err(_) => {
						warn!("Can't decode error: sp_runtime::DispatchError does not match known formats");
						return DispatchError::Other(bytes.to_vec())
					},
				};
				CurrentModuleError { index: old_e.index, error: [old_e.error, 0, 0, 0] }
			},
		};

		let error_details = match metadata.error(err.index, err.error[0]) {
			Ok(details) => details,
			Err(_) => {
				warn!("Can't decode error: sp_runtime::DispatchError::Module details do not match known information");
				return DispatchError::Other(bytes.to_vec())
			},
		};

		DispatchError::Module(ModuleError {
			pallet: error_details.pallet().to_string(),
			error: error_details.error().to_string(),
			description: error_details.docs().to_vec(),
			error_data: ModuleErrorData { pallet_index: err.index, error: err.error },
		})
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

/// Details about a module error that has occurred.
#[derive(Clone, Debug)]
pub struct ModuleError {
	/// The name of the pallet that the error came from.
	pub pallet: String,
	/// The name of the error.
	pub error: String,
	/// A description of the error.
	pub description: Vec<String>,
	/// A byte representation of the error.
	pub error_data: ModuleErrorData,
}

/// The error details about a module error that has occurred.
///
/// **Note**: Structure used to obtain the underlying bytes of a ModuleError.
#[derive(Clone, Debug)]
pub struct ModuleErrorData {
	/// Index of the pallet that the error came from.
	pub pallet_index: u8,
	/// Raw error bytes.
	pub error: [u8; 4],
}

impl ModuleErrorData {
	/// Obtain the error index from the underlying byte data.
	pub fn error_index(&self) -> u8 {
		// Error index is utilized as the first byte from the error array.
		self.error[0]
	}
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
