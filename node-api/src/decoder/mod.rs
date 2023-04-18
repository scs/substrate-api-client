// This file was taken from scale-value (Parity Technologies (UK))
// https://github.com/paritytech/scale-value/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! Decode helper.
//! It was not possible to take the scale-value as crate, because it's not no_std compatible.
//! Based on https://github.com/paritytech/scale-value/blob/4d30d609dc86cbcf102385bc34ef2c01b8c9bbb1/src/scale_impls/decode.rs

mod bit_sequence;
mod decode;
mod encode;
mod value;

use crate::alloc::vec::Vec;
use core::{fmt::Display, hash::Hash};
use derive_more::From;

pub use bit_sequence::BitSequenceError;
pub use decode::{decode_value_as_type, DecodeError};
pub use encode::{encode_value_as_type, EncodeError};
pub use scale_info::PortableRegistry;
pub use value::*;

/// The portable version of [`scale_info::Type`]
type ScaleType = scale_info::Type<scale_info::form::PortableForm>;

/// The portable version of a [`scale_info`] type ID.
type ScaleTypeId = scale_info::interner::UntrackedSymbol<core::any::TypeId>; // equivalent to: <scale_info::form::PortableForm as scale_info::form::Form>::Type;

/// The portable version of [`scale_info::TypeDef`]
type ScaleTypeDef = scale_info::TypeDef<scale_info::form::PortableForm>;

/// This represents the ID of a type found in the metadata. A scale info type representation can
/// be converted into this, and we get this back directly when decoding types into Values.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub struct TypeId(u32);

impl TypeId {
	/// Return the u32 ID expected by a PortableRegistry.
	pub(crate) fn id(self) -> u32 {
		self.0
	}
}

impl Display for TypeId {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl From<ScaleTypeId> for TypeId {
	fn from(id: ScaleTypeId) -> Self {
		TypeId(id.id)
	}
}

impl From<&ScaleTypeId> for TypeId {
	fn from(id: &ScaleTypeId) -> Self {
		TypeId(id.id)
	}
}

impl From<&TypeId> for TypeId {
	fn from(id: &TypeId) -> Self {
		*id
	}
}

/// Encoding and decoding SCALE bytes into a [`crate::Value`].
///
/// # Exmaple
///
/// Given some known metadata type ID, encode and desome some [`crate::Value`]
/// to SCALE bytes.
///
/// ```rust
/// # fn make_type<T: scale_info::TypeInfo + 'static>() -> (u32, scale_info::PortableRegistry) {
/// #     let m = scale_info::MetaType::new::<T>();
/// #     let mut types = scale_info::Registry::new();
/// #     let id = types.register_type(&m);
/// #     let portable_registry: scale_info::PortableRegistry = types.into();
/// #     (id.id(), portable_registry)
/// # }
/// # let (type_id, registry) = make_type::<Foo>();
/// use ac_node_api::decoder::Value;
///
/// // Imagine we have a `registry` (of type [`scale_info::PortableRegistry`]) containing this type,
/// // and a `type_id` (a `u32`) pointing to it in the registry.
/// #[derive(scale_info::TypeInfo)]
/// enum Foo {
///     A { is_valid: bool, name: String }
/// }
///
/// // Given that, we can encode/decode something with that shape to/from SCALE bytes:
/// let value = Value::named_variant("A", vec![
///     ("is_valid".into(), Value::bool(true)),
///     ("name".into(), Value::string("James")),
/// ]);
///
/// // Encode the Value to bytes:
/// let mut bytes = Vec::new();
/// ac_node_api::decoder::encode_as_type(value.clone(), type_id, &registry, &mut bytes).unwrap();
///
/// // Decode the bytes back into a matching Value.
/// // This value contains contextual information about which type was used
/// // to decode each part of it, which we can throw away with `.remove_context()`.
/// let new_value = ac_node_api::decoder::decode_as_type(&mut &*bytes, type_id, &registry).unwrap();
///
/// assert_eq!(value, new_value.remove_context());
/// ```

/// Attempt to decode some SCALE encoded bytes into a value, by providing a pointer
/// to the bytes (which will be moved forwards as bytes are used in the decoding),
/// a type ID, and a type registry from which we'll look up the relevant type information.
pub fn decode_as_type<Id: Into<TypeId>>(
	data: &mut &[u8],
	ty_id: Id,
	types: &PortableRegistry,
) -> Result<value::Value<TypeId>, DecodeError> {
	crate::decoder::decode_value_as_type(data, ty_id, types)
}

/// Attempt to encode some [`crate::Value<T>`] into SCALE bytes, by providing a pointer to the
/// type ID that we'd like to encode it as, a type registry from which we'll look
/// up the relevant type information, and a buffer to encode the bytes to.
pub fn encode_as_type<T, Id: Into<TypeId>>(
	value: crate::decoder::Value<T>,
	ty_id: Id,
	types: &PortableRegistry,
	buf: &mut Vec<u8>,
) -> Result<(), EncodeError<T>> {
	crate::decoder::encode_value_as_type(value, ty_id, types, buf)
}
