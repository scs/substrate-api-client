// This file was taken from scale-value (Parity Technologies (UK))
// https://github.com/paritytech/scale-value/
// And was adapted by Supercomputing Systems AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! Based on https://github.com/paritytech/scale-value/blob/430bfaf8f302dfcfc45d8d63c687628fd9b7fc25/src/scale_impls/encode.rs

use crate::scale_value::{Composite, Primitive, Value, ValueDef, Variant};
use alloc::{string::ToString, vec::Vec};
use codec::{Compact, Encode};
use scale_encode::{
	error::{ErrorKind, Kind},
	Composite as EncodeComposite, EncodeAsFields, EncodeAsType, Error, FieldIter,
	Variant as EncodeVariant,
};
use scale_info::{PortableRegistry, TypeDef};

pub use scale_encode::Error as EncodeError;

impl<T> EncodeAsType for Value<T> {
	fn encode_as_type_to(
		&self,
		type_id: u32,
		types: &PortableRegistry,
		out: &mut Vec<u8>,
	) -> Result<(), Error> {
		match &self.value {
			ValueDef::Composite(val) => encode_composite(val, type_id, types, out),
			ValueDef::Variant(val) => encode_variant(val, type_id, types, out),
			ValueDef::Primitive(val) => encode_primitive(val, type_id, types, out),
			ValueDef::BitSequence(_val) => unimplemented!(),
		}
	}
}

impl<T> EncodeAsFields for Value<T> {
	fn encode_as_fields_to(
		&self,
		fields: &mut dyn FieldIter<'_>,
		types: &PortableRegistry,
		out: &mut Vec<u8>,
	) -> Result<(), Error> {
		match &self.value {
			ValueDef::Composite(composite) => composite.encode_as_fields_to(fields, types, out),
			_ => Err(Error::custom("Cannot encode non-composite Value shape into fields")),
		}
	}
}

impl<T> EncodeAsFields for Composite<T> {
	fn encode_as_fields_to(
		&self,
		fields: &mut dyn FieldIter<'_>,
		types: &PortableRegistry,
		out: &mut Vec<u8>,
	) -> Result<(), Error> {
		match self {
			Composite::Named(vals) => {
				let keyvals =
					vals.iter().map(|(key, val)| (Some(&**key), val as &dyn EncodeAsType));
				EncodeComposite(keyvals).encode_as_fields_to(fields, types, out)
			},
			Composite::Unnamed(vals) => {
				let vals = vals.iter().map(|val| (None, val as &dyn EncodeAsType));
				EncodeComposite(vals).encode_as_fields_to(fields, types, out)
			},
		}
	}
}

// A scale-value composite type can represent sequences, arrays, composites and tuples. `scale_encode`'s Composite helper
// can't handle encoding to sequences/arrays. However, we can encode safely into sequences here because we can inspect the
// values we have and more safely skip newtype wrappers without also skipping through types that might represent 1-value
// sequences/arrays for instance.
fn encode_composite<T>(
	value: &Composite<T>,
	mut type_id: u32,
	types: &PortableRegistry,
	out: &mut Vec<u8>,
) -> Result<(), Error> {
	// Encode our composite Value as-is (pretty much; we will try to
	// unwrap the Value only if we need primitives).
	fn do_encode_composite<T>(
		value: &Composite<T>,
		type_id: u32,
		types: &PortableRegistry,
		out: &mut Vec<u8>,
	) -> Result<(), Error> {
		let ty = types
			.resolve(type_id)
			.ok_or_else(|| Error::new(ErrorKind::TypeNotFound(type_id)))?;
		match &ty.type_def {
			TypeDef::Tuple(_) | TypeDef::Composite(_) => match value {
				Composite::Named(vals) => {
					let keyvals =
						vals.iter().map(|(key, val)| (Some(&**key), val as &dyn EncodeAsType));
					EncodeComposite(keyvals).encode_as_type_to(type_id, types, out)
				},
				Composite::Unnamed(vals) => {
					let vals = vals.iter().map(|val| (None, val as &dyn EncodeAsType));
					EncodeComposite(vals).encode_as_type_to(type_id, types, out)
				},
			},
			TypeDef::Sequence(seq) => {
				// sequences start with compact encoded length:
				Compact(value.len() as u32).encode_to(out);
				match value {
					Composite::Named(named_vals) =>
						for (name, val) in named_vals {
							val.encode_as_type_to(seq.type_param.id, types, out)
								.map_err(|e| e.at_field(name.to_string()))?;
						},
					Composite::Unnamed(vals) =>
						for (idx, val) in vals.iter().enumerate() {
							val.encode_as_type_to(seq.type_param.id, types, out)
								.map_err(|e| e.at_idx(idx))?;
						},
				}
				Ok(())
			},
			TypeDef::Array(array) => {
				let arr_ty = array.type_param.id;
				if value.len() != array.len as usize {
					return Err(Error::new(ErrorKind::WrongLength {
						actual_len: value.len(),
						expected_len: array.len as usize,
					}))
				}

				for (idx, val) in value.values().enumerate() {
					val.encode_as_type_to(arr_ty, types, out).map_err(|e| e.at_idx(idx))?;
				}
				Ok(())
			},
			TypeDef::BitSequence(_seq) => unimplemented!(),
			// For other types, skip our value past a 1-value composite and try again, else error.
			_ => {
				let mut values = value.values();
				match (values.next(), values.next()) {
					// Exactly one value:
					(Some(value), None) => value.encode_as_type_to(type_id, types, out),
					// Some other number of values:
					_ => Err(Error::new(ErrorKind::WrongShape {
						actual: Kind::Tuple,
						expected: type_id,
					})),
				}
			},
		}
	}

	// First, try and encode everything as-is, only writing to the output
	// byte if the encoding is actually successful. This means that if the
	// Value provided matches the structure of the TypeInfo exactly, things
	// should always work.
	let original_error = {
		let mut temp_out = Vec::new();
		match do_encode_composite(value, type_id, types, &mut temp_out) {
			Ok(()) => {
				out.extend_from_slice(&temp_out);
				return Ok(())
			},
			Err(e) => e,
		}
	};

	// Next, unwrap any newtype wrappers from our TypeInfo and try again. If we
	// can unwrap, then try to encode our Value to this immediately (this will work
	// if the Value provided already ignored all newtype wrappers). If we have nothing
	// to unwrap then ignore this extra encode attempt.
	{
		let inner_type_id = find_single_entry_with_same_repr(type_id, types);
		if inner_type_id != type_id {
			let mut temp_out = Vec::new();
			if let Ok(()) = do_encode_composite(value, inner_type_id, types, &mut temp_out) {
				out.extend_from_slice(&temp_out);
				return Ok(())
			}
			type_id = inner_type_id;
		}
	}

	// Now, start peeling layers off our Value type in case some newtype wrappers
	// were provided. We do this one layer at a time because it's difficult or
	// impossible to know how to line values up with TypeInfo, so we can't just
	// strip lots of layers from the Value straight away. We continue to ignore
	// any errors here and will always return the original_error if we can't encode.
	// Everything past the original attempt is just trying to be flexible, anyway.
	while let Some(value) = get_only_value_from_composite(value) {
		let mut temp_out = Vec::new();
		if let Ok(()) = value.encode_as_type_to(type_id, types, &mut temp_out) {
			out.extend_from_slice(&temp_out);
			return Ok(())
		}
	}

	// return the original error we got back if none of the above is succcessful.
	Err(original_error)
}

// skip into the target type past any newtype wrapper like things:
fn find_single_entry_with_same_repr(type_id: u32, types: &PortableRegistry) -> u32 {
	let Some(ty) = types.resolve(type_id) else {
        return type_id
    };
	match &ty.type_def {
		TypeDef::Tuple(tuple) if tuple.fields.len() == 1 =>
			find_single_entry_with_same_repr(tuple.fields[0].id, types),
		TypeDef::Composite(composite) if composite.fields.len() == 1 =>
			find_single_entry_with_same_repr(composite.fields[0].ty.id, types),
		TypeDef::Array(arr) if arr.len == 1 =>
			find_single_entry_with_same_repr(arr.type_param.id, types),
		_ => type_id,
	}
}

// if the composite type has exactly one value, return that Value, else return None.
fn get_only_value_from_composite<T>(value: &'_ Composite<T>) -> Option<&'_ Value<T>> {
	let mut values = value.values();
	match (values.next(), values.next()) {
		(Some(value), None) => Some(value),
		_ => None,
	}
}

fn encode_variant<T>(
	value: &Variant<T>,
	type_id: u32,
	types: &PortableRegistry,
	out: &mut Vec<u8>,
) -> Result<(), Error> {
	match &value.values {
		Composite::Named(vals) => {
			let keyvals = vals.iter().map(|(key, val)| (Some(&**key), val as &dyn EncodeAsType));
			EncodeVariant { name: &value.name, fields: EncodeComposite(keyvals) }
				.encode_as_type_to(type_id, types, out)
		},
		Composite::Unnamed(vals) => {
			let vals = vals.iter().map(|val| (None, val as &dyn EncodeAsType));
			EncodeVariant { name: &value.name, fields: EncodeComposite(vals) }
				.encode_as_type_to(type_id, types, out)
		},
	}
}

fn encode_primitive(
	value: &Primitive,
	type_id: u32,
	types: &PortableRegistry,
	bytes: &mut Vec<u8>,
) -> Result<(), Error> {
	match value {
		Primitive::Bool(val) => val.encode_as_type_to(type_id, types, bytes),
		Primitive::Char(val) => val.encode_as_type_to(type_id, types, bytes),
		Primitive::String(val) => val.encode_as_type_to(type_id, types, bytes),
		Primitive::U128(val) => val.encode_as_type_to(type_id, types, bytes),
		Primitive::I128(val) => val.encode_as_type_to(type_id, types, bytes),
		Primitive::U256(val) => val.encode_as_type_to(type_id, types, bytes),
		Primitive::I256(val) => val.encode_as_type_to(type_id, types, bytes),
	}
}
