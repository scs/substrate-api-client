// This file was taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! General node-api Error and Substrate DispatchError implementation.

use crate::{
	alloc::{
		borrow::Cow,
		format,
		string::{String, ToString},
		vec::Vec,
	},
	metadata::Metadata,
};

use codec::{Decode, Encode};
use core::fmt::Debug;
use derive_more::From;
use log::*;
use scale_info::TypeDef;

// Re-expose the errors we use from other crates here:
pub use crate::{
	metadata::{InvalidMetadataError, MetadataError},
	scale_value::{DecodeError, EncodeError},
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

/// An error dispatching a transaction. See Substrate DispatchError
//https://github.com/paritytech/substrate/blob/890451221db37176e13cb1a306246f02de80590a/primitives/runtime/src/lib.rs#L524
#[derive(Debug, From)]
pub enum DispatchError {
	/// Some error occurred.
	Other(Vec<u8>),
	/// Failed to lookup some data.
	CannotLookup,
	/// A bad origin.
	BadOrigin,
	/// A custom error in a module.
	Module(ModuleError),
	/// At least one consumer is remaining so the account cannot be destroyed.
	ConsumerRemaining,
	/// There are no providers so the account cannot be created.
	NoProviders,
	/// There are too many consumers so the account cannot be created.
	TooManyConsumers,
	/// An error to do with tokens.
	Token(TokenError),
	/// An arithmetic error.
	Arithmetic(ArithmeticError),
	/// The number of transactional layers has been reached, or we are not in a transactional layer.
	Transactional(TransactionalError),
	/// Resources exhausted, e.g. attempt to read/write data which is too large to manipulate.
	Exhausted,
	/// The state is corrupt; this is generally not going to fix itself.
	Corruption,
	/// Some resource (e.g. a preimage) is unavailable right now. This might fix itself later.
	Unavailable,
}

impl DispatchError {
	/// Attempt to decode a runtime DispatchError
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

		let variant_name =
			variant.variants.iter().find(|v| v.index == bytes[0]).map(|v| v.name.as_str());
		let name = match variant_name {
			Some(name) => name,
			None => {
				warn!("Can't decode error: sp_runtime::DispatchError does not have a name variant");
				return DispatchError::Other(bytes.into_owned())
			},
		};

		if bytes.len() < 2 {
			warn!(
				"Can't decode error: sp_runtime::DispatchError because it contains too few bytes"
			);
			return DispatchError::Other(bytes.into_owned())
		}
		// The remaining bytes are the specific error to decode:
		let mut specific_bytes = &bytes[1..];

		match name {
			"Module" => Self::decode_module_error(specific_bytes, metadata), // We apply custom logic to transform the module error into the outward facing version
			"Token" => {
				let token_error = match TokenError::decode(&mut specific_bytes) {
					Ok(err) => err,
					Err(_) => {
						warn!("Can't decode token error: TokenError does not match known formats");
						return DispatchError::Other(bytes.to_vec())
					},
				};
				DispatchError::Token(token_error)
			},
			"Arithmetic" => {
				let arithmetic_error = match ArithmeticError::decode(&mut specific_bytes) {
					Ok(err) => err,
					Err(_) => {
						warn!("Can't decode arithmetic error: ArithmeticError does not match known formats");
						return DispatchError::Other(bytes.to_vec())
					},
				};
				DispatchError::Arithmetic(arithmetic_error)
			},
			"Transactional" => {
				let error = match TransactionalError::decode(&mut specific_bytes) {
					Ok(err) => err,
					Err(_) => {
						warn!("Can't decode transactional error: TransactionalError does not match known formats");
						return DispatchError::Other(bytes.to_vec())
					},
				};
				DispatchError::Transactional(error)
			},
			"CannotLookup" => DispatchError::CannotLookup,
			"BadOrigin" => DispatchError::BadOrigin,
			"ConsumerRemaining" => DispatchError::ConsumerRemaining,
			"NoProviders" => DispatchError::NoProviders,
			"TooManyConsumers" => DispatchError::TooManyConsumers,
			"Exhausted" => DispatchError::Exhausted,
			"Corruption" => DispatchError::Corruption,
			"Unavailable" => DispatchError::Unavailable,
			_ => {
				warn!("Can't decode runtime dispatch error: sp_runtime::DispatchError  does not match known formats");
				DispatchError::Other(bytes.into_owned())
			},
		}
	}

	/// ModuleError is a bit special; we want to support being decoded from either
	/// a legacy format of 2 bytes, or a newer format of 5 bytes. So, just grab the bytes
	/// out when decoding to manually work with them.
	fn decode_module_error(mut bytes: &[u8], metadata: &Metadata) -> Self {
		// The oldest and second oldest type of error decode to this shape.
		// The old version is 2 bytes; a pallet and error index.
		#[derive(Decode)]
		struct LegacyModuleError {
			index: u8,
			error: u8,
		}

		// The newer case expands the error for forward compat:
		// The new version is 5 bytes; a pallet and error index and then 3 extra bytes.
		#[derive(Decode)]
		struct CurrentModuleError {
			index: u8,
			error: [u8; 4],
		}

		// try to decode into the new shape, or the old if that doesn't work
		let err = match CurrentModuleError::decode(&mut bytes) {
			Ok(e) => e,
			Err(_) => {
				let old_e = match LegacyModuleError::decode(&mut bytes) {
					Ok(err) => err,
					Err(_) => {
						warn!("Can't decode module error: sp_runtime::DispatchError does not match known formats");
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

/// An error relating to tokens when dispatching a transaction.
//https://github.com/paritytech/substrate/blob/890451221db37176e13cb1a306246f02de80590a/primitives/runtime/src/lib.rs#L607
#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub enum TokenError {
	/// Funds are unavailable.
	FundsUnavailable,
	/// Some part of the balance gives the only provider reference to the account and thus cannot be (re)moved.
	OnlyProvider,
	/// Account cannot exist with the funds that would be given.
	BelowMinimum,
	/// Account cannot be created.
	CannotCreate,
	/// The asset in question is unknown.
	UnknownAsset,
	/// Funds exist but are frozen.
	Frozen,
	/// Operation is not supported by the asset.
	Unsupported,
	/// Account cannot be created for a held balance.
	CannotCreateHold,
	/// Withdrawal would cause unwanted loss of account.
	NotExpendable,
}

/// An error relating to arithmetic when dispatching a transaction.
// https://github.com/paritytech/substrate/blob/890451221db37176e13cb1a306246f02de80590a/primitives/arithmetic/src/lib.rs#L59
#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub enum ArithmeticError {
	/// Underflow.
	Underflow,
	/// Overflow.
	Overflow,
	/// Division by zero.
	DivisionByZero,
}

/// An error relating to the transactional layers when dispatching a transaction.
// https://github.com/paritytech/substrate/blob/890451221db37176e13cb1a306246f02de80590a/primitives/runtime/src/lib.rs#L496
#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub enum TransactionalError {
	/// Too many transactional layers have been spawned.
	LimitReached,
	/// A transactional layer was expected, but does not exist.
	NoLayer,
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
