// This file was taken from scale-value (Parity Technologies (UK))
// https://github.com/paritytech/scale-value/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! Decode helpers.
//! It was not possible to take the scale-value as crate, because it's not no_std compatible.
//! Based on https://github.com/paritytech/scale-value/blob/430bfaf8f302dfcfc45d8d63c687628fd9b7fc25/src/lib.rs

mod decode;
mod encode;
mod value;

// The value definition.
pub use value::{BitSequence, Composite, Primitive, Value, ValueDef, Variant};

/// A type ID which can be resolved into a type given a [`scale_info::PortableRegistry`].
pub type TypeId = u32;

pub use scale::*;
pub use scale_info::PortableRegistry;

pub mod scale {
	use super::TypeId;
	pub use super::{
		decode::{DecodeError, DecodeValueVisitor},
		encode::EncodeError,
	};
	use alloc::vec::Vec;
	use scale_encode::EncodeAsType;
	pub use scale_info::PortableRegistry;

	/// Attempt to decode some SCALE encoded bytes into a value, by providing a pointer
	/// to the bytes (which will be moved forwards as bytes are used in the decoding),
	/// a type ID, and a type registry from which we'll look up the relevant type information.
	pub fn decode_as_type(
		data: &mut &[u8],
		ty_id: TypeId,
		types: &PortableRegistry,
	) -> Result<super::Value<TypeId>, DecodeError> {
		crate::scale_value::decode::decode_value_as_type(data, ty_id, types)
	}

	/// Attempt to encode some [`crate::Value<T>`] into SCALE bytes, by providing a pointer to the
	/// type ID that we'd like to encode it as, a type registry from which we'll look
	/// up the relevant type information, and a buffer to encode the bytes to.
	pub fn encode_as_type<T: Clone>(
		value: &super::Value<T>,
		ty_id: TypeId,
		types: &PortableRegistry,
		buf: &mut Vec<u8>,
	) -> Result<(), EncodeError> {
		value.encode_as_type_to(ty_id, types, buf)
	}
}
