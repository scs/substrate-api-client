// This file was taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! Substrate Dispatch Error representation.

use super::{Error, MetadataError};
use crate::metadata::Metadata;
use alloc::{
	borrow::Cow,
	string::{String, ToString},
	vec::Vec,
};
use codec::{Decode, Encode};
use core::fmt::Debug;
use derive_more::From;
use log::*;
use scale_decode::{visitor::DecodeAsTypeResult, DecodeAsType};

/// An error dispatching a transaction. See Substrate DispatchError
// https://github.com/paritytech/polkadot-sdk/blob/0c5dcca9e3cef6b2f456fccefd9f6c5e43444053/substrate/primitives/runtime/src/lib.rs#L561-L598
#[derive(Debug, From, PartialEq, Eq)]
pub enum DispatchError {
	/// Some error occurred.
	Other,
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
	/// Root origin is not allowed.
	RootNotAllowed,
}

impl DispatchError {
	/// Attempt to decode a runtime [`DispatchError`].
	// https://github.com/paritytech/subxt/blob/0d1cc92f27c0c6d43de16fe7276484a141149096/subxt/src/error/dispatch_error.rs#L229-L338
	pub fn decode_from<'a>(
		bytes: impl Into<Cow<'a, [u8]>>,
		metadata: &Metadata,
	) -> Result<Self, Error> {
		let bytes = bytes.into();
		let dispatch_error_ty_id =
			metadata.dispatch_error_ty().ok_or(MetadataError::DispatchErrorNotFound)?;

		// The aim is to decode our bytes into roughly this shape. This is copied from
		// `sp_runtime::DispatchError`; we need the variant names and any inner variant
		// names/shapes to line up in order for decoding to be successful.
		#[derive(DecodeAsType)]
		enum DecodedDispatchError {
			Other,
			CannotLookup,
			BadOrigin,
			Module(DecodedModuleErrorBytes),
			ConsumerRemaining,
			NoProviders,
			TooManyConsumers,
			Token(TokenError),
			Arithmetic(ArithmeticError),
			Transactional(TransactionalError),
			Exhausted,
			Corruption,
			Unavailable,
			RootNotAllowed,
		}

		// ModuleError is a bit special; we want to support being decoded from either
		// a legacy format of 2 bytes, or a newer format of 5 bytes. So, just grab the bytes
		// out when decoding to manually work with them.
		struct DecodedModuleErrorBytes(Vec<u8>);
		struct DecodedModuleErrorBytesVisitor;
		impl scale_decode::Visitor for DecodedModuleErrorBytesVisitor {
			type Error = scale_decode::Error;
			type Value<'scale, 'info> = DecodedModuleErrorBytes;
			fn unchecked_decode_as_type<'scale, 'info>(
				self,
				input: &mut &'scale [u8],
				_type_id: scale_decode::visitor::TypeId,
				_types: &'info scale_info::PortableRegistry,
			) -> DecodeAsTypeResult<Self, Result<Self::Value<'scale, 'info>, Self::Error>> {
				DecodeAsTypeResult::Decoded(Ok(DecodedModuleErrorBytes(input.to_vec())))
			}
		}
		impl scale_decode::IntoVisitor for DecodedModuleErrorBytes {
			type Visitor = DecodedModuleErrorBytesVisitor;
			fn into_visitor() -> Self::Visitor {
				DecodedModuleErrorBytesVisitor
			}
		}

		// Decode into our temporary error:
		let decoded_dispatch_err = DecodedDispatchError::decode_with_metadata(
			&mut &*bytes,
			dispatch_error_ty_id,
			&metadata,
		)?;

		// Convert into the outward-facing error, mainly by handling the Module variant.
		let dispatch_error = match decoded_dispatch_err {
			// Mostly we don't change anything from our decoded to our outward-facing error:
			DecodedDispatchError::Other => DispatchError::Other,
			DecodedDispatchError::CannotLookup => DispatchError::CannotLookup,
			DecodedDispatchError::BadOrigin => DispatchError::BadOrigin,
			DecodedDispatchError::ConsumerRemaining => DispatchError::ConsumerRemaining,
			DecodedDispatchError::NoProviders => DispatchError::NoProviders,
			DecodedDispatchError::TooManyConsumers => DispatchError::TooManyConsumers,
			DecodedDispatchError::Token(val) => DispatchError::Token(val),
			DecodedDispatchError::Arithmetic(val) => DispatchError::Arithmetic(val),
			DecodedDispatchError::Transactional(val) => DispatchError::Transactional(val),
			DecodedDispatchError::Exhausted => DispatchError::Exhausted,
			DecodedDispatchError::Corruption => DispatchError::Corruption,
			DecodedDispatchError::Unavailable => DispatchError::Unavailable,
			DecodedDispatchError::RootNotAllowed => DispatchError::RootNotAllowed,
			// But we apply custom logic to transform the module error into the outward facing version:
			DecodedDispatchError::Module(module_bytes) => {
				let module_bytes = module_bytes.0;

				// The old version is 2 bytes; a pallet and error index.
				// The new version is 5 bytes; a pallet and error index and then 3 extra bytes.
				let bytes = if module_bytes.len() == 2 {
					[module_bytes[0], module_bytes[1], 0, 0, 0]
				} else if module_bytes.len() == 5 {
					[
						module_bytes[0],
						module_bytes[1],
						module_bytes[2],
						module_bytes[3],
						module_bytes[4],
					]
				} else {
					warn!("Can't decode error sp_runtime::DispatchError: bytes do not match known shapes");
					// Return _all_ of the bytes; every "unknown" return should be consistent.
					return Err(Error::Unknown(bytes.to_vec()));
				};

				// And return our outward-facing version:
				DispatchError::Module(ModuleError { metadata, bytes })
			},
		};

		Ok(dispatch_error)
	}
}

/// An error relating to tokens when dispatching a transaction.
// https://github.com/paritytech/polkadot-sdk/blob/0c5dcca9e3cef6b2f456fccefd9f6c5e43444053/substrate/primitives/runtime/src/lib.rs#L646-L671
#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, DecodeAsType)]
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
	/// Account cannot receive the assets.
	Blocked,
}

/// An error relating to arithmetic when dispatching a transaction.
// https://github.com/paritytech/polkadot-sdk/blob/0c5dcca9e3cef6b2f456fccefd9f6c5e43444053/substrate/primitives/arithmetic/src/lib.rs#L61-L71
#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, DecodeAsType)]
pub enum ArithmeticError {
	/// Underflow.
	Underflow,
	/// Overflow.
	Overflow,
	/// Division by zero.
	DivisionByZero,
}

/// An error relating to thr transactional layers when dispatching a transaction.
// https://github.com/paritytech/polkadot-sdk/blob/0c5dcca9e3cef6b2f456fccefd9f6c5e43444053/substrate/primitives/runtime/src/lib.rs#L536-L544
#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, DecodeAsType)]
pub enum TransactionalError {
	/// Too many transactional layers have been spawned.
	LimitReached,
	/// A transactional layer was expected, but does not exist.
	NoLayer,
}

/// Details about a module error that has occurred.
// https://github.com/paritytech/subxt/blob/0d1cc92f27c0c6d43de16fe7276484a141149096/subxt/src/error/dispatch_error.rs#L167-L226
#[derive(Clone)]
pub struct ModuleError {
	metadata: Metadata,
	/// Bytes representation:
	///  - `bytes[0]`:   pallet index
	///  - `bytes[1]`:   error index
	///  - `bytes[2..]`: 3 bytes specific for the module error
	bytes: [u8; 5],
}

impl PartialEq for ModuleError {
	fn eq(&self, other: &Self) -> bool {
		// A module error is the same if the raw underlying details are the same.
		self.bytes == other.bytes
	}
}

impl Eq for ModuleError {}

/// Custom `Debug` implementation, ignores the very large `metadata` field, using it instead (as
/// intended) to resolve the actual pallet and error names. This is much more useful for debugging.
impl Debug for ModuleError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
		let details = self.details_string();
		write!(f, "ModuleError(<{details}>)")
	}
}

impl core::fmt::Display for ModuleError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		let details = self.details_string();
		write!(f, "{details}")
	}
}

impl ModuleError {
	/// Return more details about this error.
	pub fn details(&self) -> Result<ModuleErrorDetails, MetadataError> {
		let pallet = self.metadata.pallet_by_index_err(self.pallet_index())?;
		let variant = pallet
			.error_variant_by_index(self.error_index())
			.ok_or_else(|| MetadataError::VariantIndexNotFound(self.error_index()))?;

		Ok(ModuleErrorDetails { pallet, variant })
	}

	/// Return a formatted string of the resolved error details for debugging/display purposes.
	pub fn details_string(&self) -> String {
		match self.details() {
			Ok(details) => format!(
				"{pallet_name}::{variant_name}",
				pallet_name = details.pallet.name(),
				variant_name = details.variant.name,
			),
			Err(_) => format!(
				"Unknown pallet error '{bytes:?}' (pallet and error details cannot be retrieved)",
				bytes = self.bytes
			),
		}
	}

	/// Return the underlying module error data that was decoded.
	pub fn bytes(&self) -> [u8; 5] {
		self.bytes
	}

	/// Obtain the pallet index from the underlying byte data.
	pub fn pallet_index(&self) -> u8 {
		self.bytes[0]
	}

	/// Obtain the error index from the underlying byte data.
	pub fn error_index(&self) -> u8 {
		self.bytes[1]
	}

	// 	/// Attempts to decode the ModuleError into the top outer Error enum.
	// 	pub fn as_root_error<E: DecodeAsType>(&self) -> Result<E, Error> {
	// 		let decoded = E::decode_as_type(
	// 			&mut &self.bytes[..],
	// 			self.metadata.outer_enums().error_enum_ty(),
	// 			self.metadata.types(),
	// 		)?;
	//
	// 		Ok(decoded)
	// 	}
}

/// Details about the module error.
pub struct ModuleErrorDetails<'a> {
	/// The pallet that the error is in
	pub pallet: crate::metadata::PalletMetadata<'a>,
	/// The variant representing the error
	pub variant: &'a scale_info::Variant<scale_info::form::PortableForm>,
}
