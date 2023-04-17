// This file was taken from scale-value (Parity Technologies (UK))
// https://github.com/paritytech/scale-value/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

use super::ScaleTypeDef as TypeDef;
use crate::alloc::{format, string::String};
use scale_info::{form::PortableForm, PortableRegistry, TypeDefBitSequence, TypeDefPrimitive};

/// An error that can occur when we try to encode or decode to a SCALE bit sequence type.
#[derive(Debug, Clone, PartialEq)]
pub enum BitSequenceError {
	/// The registry did not contain the bit order type listed.
	BitOrderTypeNotFound(u32),
	/// The registry did not contain the bit store type listed.
	BitStoreTypeNotFound(u32),
	/// The bit order type did not have a valid identifier/name.
	NoBitOrderIdent,
	/// The bit store type that we found was not what we expected (a primitive u8/u16/u32/u64).
	StoreTypeNotSupported(String),
	/// The bit order type name that we found was not what we expected ("Lsb0" or "Msb0").
	OrderTypeNotSupported(String),
}

/// Obtain details about a bit sequence.
pub fn get_bitsequence_details(
	ty: &TypeDefBitSequence<PortableForm>,
	types: &PortableRegistry,
) -> Result<(BitOrderTy, BitStoreTy), BitSequenceError> {
	let bit_store_ty = ty.bit_store_type.id;
	let bit_order_ty = ty.bit_order_type.id;

	// What is the backing store type expected?
	let bit_store_def = &types
		.resolve(bit_store_ty)
		.ok_or(BitSequenceError::BitStoreTypeNotFound(bit_store_ty))?
		.type_def;

	// What is the bit order type expected?
	let bit_order_def = types
		.resolve(bit_order_ty)
		.ok_or(BitSequenceError::BitOrderTypeNotFound(bit_order_ty))?
		.path
		.ident()
		.ok_or(BitSequenceError::NoBitOrderIdent)?;

	let bit_order_out = match bit_store_def {
		TypeDef::Primitive(TypeDefPrimitive::U8) => Some(BitOrderTy::U8),
		TypeDef::Primitive(TypeDefPrimitive::U16) => Some(BitOrderTy::U16),
		TypeDef::Primitive(TypeDefPrimitive::U32) => Some(BitOrderTy::U32),
		#[cfg(target_pointer_width = "64")]
		TypeDef::Primitive(TypeDefPrimitive::U64) => Some(BitOrderTy::U64),
		_ => None,
	}
	.ok_or_else(|| BitSequenceError::OrderTypeNotSupported(format!("{bit_store_def:?}")))?;

	let bit_store_out = match &*bit_order_def {
		"Lsb0" => Some(BitStoreTy::Lsb0),
		"Msb0" => Some(BitStoreTy::Msb0),
		_ => None,
	}
	.ok_or(BitSequenceError::StoreTypeNotSupported(bit_order_def))?;

	Ok((bit_order_out, bit_store_out))
}

#[derive(Copy, Clone, PartialEq)]
pub enum BitStoreTy {
	Lsb0,
	Msb0,
}

#[derive(Copy, Clone, PartialEq)]
pub enum BitOrderTy {
	U8,
	U16,
	U32,
	#[cfg(target_pointer_width = "64")]
	U64,
}
