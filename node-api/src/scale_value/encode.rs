// This file was taken from scale-value (Parity Technologies (UK))
// https://github.com/paritytech/scale-value/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

use super::{
	value::{Composite, Primitive, Value, ValueDef, Variant},
	TypeId,
};
use crate::alloc::{string::String, vec::Vec};
use codec::{Compact, Encode};
use scale_info::{
	form::PortableForm, Field, PortableRegistry, TypeDef, TypeDefArray, TypeDefCompact,
	TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeDefVariant,
};

/// An error encoding a [`Value`] into SCALE bytes.
#[derive(Debug, Clone, PartialEq)]
pub enum EncodeError<T> {
	/// The composite type we're trying to encode is the wrong length for the type we're trying to encode it into.
	CompositeIsWrongLength {
		/// The composite value that is the wrong length.
		actual: Composite<T>,
		/// The type we're trying to encode it into.
		expected: TypeId,
		/// The length we're expecting our composite type to be to encode properly.
		expected_len: usize,
	},
	/// The composite is expected to contain named or unnamed values to encode properly, and the opposite is true.
	CompositeIsWrongShape {
		/// The composite value that is the wrong shape.
		actual: Composite<T>,
		/// The type we're trying to encode it into.
		expected: TypeId,
	},
	/// The variant we're trying to encode was not found in the type we're encoding into.
	VariantNotFound {
		/// The variant type we're trying to encode.
		actual: Variant<T>,
		/// The type we're trying to encode it into.
		expected: TypeId,
	},
	/// The variant or composite field we're trying to encode is not present in the type we're encoding into.
	CompositeFieldIsMissing {
		/// The name of the composite field we can't find.
		missing_field_name: String,
		/// The type we're trying to encode this into.
		expected: TypeId,
	},
	/// The type we're trying to encode into cannot be found in the type registry provided.
	TypeIdNotFound(TypeId),
	/// The [`Value`] type we're trying to encode is not the correct shape for the type we're trying to encode it into.
	WrongShape {
		/// The value we're trying to encode.
		actual: Value<T>,
		/// The type we're trying to encode it into.
		expected: TypeId,
	},
	/// The type ID given is supposed to be compact encoded, but this is not possible to do automatically.
	CannotCompactEncode(TypeId),
}

/// Attempt to SCALE Encode a Value according to the [`TypeId`] and
/// [`PortableRegistry`] provided.
pub fn encode_value_as_type<T, Id: Into<TypeId>>(
	value: Value<T>,
	ty_id: Id,
	types: &PortableRegistry,
	bytes: &mut Vec<u8>,
) -> Result<(), EncodeError<T>> {
	let ty_id = ty_id.into();
	let ty = types.resolve(ty_id).ok_or(EncodeError::TypeIdNotFound(ty_id))?;

	match &ty.type_def {
		TypeDef::Composite(inner) => encode_composite_value(value, ty_id, inner, types, bytes),
		TypeDef::Sequence(inner) => encode_sequence_value(value, ty_id, inner, types, bytes),
		TypeDef::Array(inner) => encode_array_value(value, ty_id, inner, types, bytes),
		TypeDef::Tuple(inner) => encode_tuple_value(value, ty_id, inner, types, bytes),
		TypeDef::Variant(inner) => encode_variant_value(value, ty_id, inner, types, bytes),
		TypeDef::Primitive(inner) => encode_primitive_value(value, ty_id, inner, bytes),
		TypeDef::Compact(inner) => encode_compact_value(value, ty_id, inner, types, bytes),
		TypeDef::BitSequence(inner) => unimplemented!(),
	}?;

	Ok(())
}

fn encode_composite_value<T>(
	value: Value<T>,
	type_id: TypeId,
	ty: &TypeDefComposite<PortableForm>,
	types: &PortableRegistry,
	bytes: &mut Vec<u8>,
) -> Result<(), EncodeError<T>> {
	match value.value {
		ValueDef::Composite(composite) =>
			encode_composite_fields(composite, &ty.fields, type_id, types, bytes),
		_ => {
			if ty.fields.len() == 1 {
				// A 1-field composite type? try encoding inner content then.
				encode_value_as_type(value, ty.fields.id(), types, bytes)
			} else {
				Err(EncodeError::WrongShape { actual: value, expected: type_id })
			}
		},
	}
}

fn encode_sequence_value<T>(
	value: Value<T>,
	type_id: TypeId,
	ty: &TypeDefSequence<PortableForm>,
	types: &PortableRegistry,
	bytes: &mut Vec<u8>,
) -> Result<(), EncodeError<T>> {
	match value.value {
		// Let's see whether our composite type is the right length,
		// and try to encode each inner value into what the sequence wants.
		ValueDef::Composite(c) => {
			// Compact encoded length comes first
			Compact(c.len() as u64).encode_to(bytes);
			let ty = ty.type_param;
			for value in c.into_values() {
				encode_value_as_type(value, ty, types, bytes)?;
			}
		},
		// As a special case, primitive U256/I256s are arrays, and may be compatible
		// with the sequence type being asked for, too.
		ValueDef::Primitive(Primitive::I256(a) | Primitive::U256(a)) => {
			// Compact encoded length comes first
			Compact(a.len() as u64).encode_to(bytes);
			let ty = ty.type_param;
			for val in a {
				if encode_value_as_type(Value::u128(val.into()), ty, types, bytes).is_err() {
					return Err(EncodeError::WrongShape { actual: value, expected: type_id })
				}
			}
		},
		_ => return Err(EncodeError::WrongShape { actual: value, expected: type_id }),
	};
	Ok(())
}

fn encode_array_value<T>(
	value: Value<T>,
	type_id: TypeId,
	ty: &TypeDefArray<PortableForm>,
	types: &PortableRegistry,
	bytes: &mut Vec<u8>,
) -> Result<(), EncodeError<T>> {
	match value.value {
		// Let's see whether our composite type is the right length,
		// and try to encode each inner value into what the array wants.
		ValueDef::Composite(c) => {
			let arr_len = ty.len as usize;
			if c.len() != arr_len {
				return Err(EncodeError::CompositeIsWrongLength {
					actual: c,
					expected: type_id,
					expected_len: arr_len,
				})
			}

			let ty = ty.type_param;
			for value in c.into_values() {
				encode_value_as_type(value, ty, types, bytes)?;
			}
		},
		// As a special case, primitive U256/I256s are arrays, and may be compatible
		// with the array type being asked for, too.
		ValueDef::Primitive(Primitive::I256(a) | Primitive::U256(a)) => {
			let arr_len = ty.len as usize;
			if a.len() != arr_len {
				return Err(EncodeError::WrongShape { actual: value, expected: type_id })
			}

			let ty = ty.type_param;
			for val in a {
				if encode_value_as_type(Value::u128(val.into()), ty, types, bytes).is_err() {
					return Err(EncodeError::WrongShape { actual: value, expected: type_id })
				}
			}
		},
		_ => return Err(EncodeError::WrongShape { actual: value, expected: type_id }),
	};
	Ok(())
}

fn encode_tuple_value<T>(
	value: Value<T>,
	type_id: TypeId,
	ty: &TypeDefTuple<PortableForm>,
	types: &PortableRegistry,
	bytes: &mut Vec<u8>,
) -> Result<(), EncodeError<T>> {
	match value.value {
		ValueDef::Composite(composite) => {
			if composite.len() != ty.fields.len() {
				return Err(EncodeError::CompositeIsWrongLength {
					actual: composite,
					expected: type_id,
					expected_len: ty.fields.len(),
				})
			}
			// We don't care whether the fields are named or unnamed
			// as long as we have the number of them that we expect..
			let field_value_pairs = ty.fields.iter().zip(composite.into_values());
			for (ty, value) in field_value_pairs {
				encode_value_as_type(value, ty, types, bytes)?;
			}
			Ok(())
		},
		_ => {
			if ty.fields.len() == 1 {
				// A 1-field tuple? try encoding inner content then.
				encode_value_as_type(value, ty.fields[0], types, bytes)
			} else {
				Err(EncodeError::WrongShape { actual: value, expected: type_id })
			}
		},
	}
}

fn encode_variant_value<T>(
	value: Value<T>,
	type_id: TypeId,
	ty: &TypeDefVariant<PortableForm>,
	types: &PortableRegistry,
	bytes: &mut Vec<u8>,
) -> Result<(), EncodeError<T>> {
	let variant = match value.value {
		ValueDef::Variant(variant) => variant,
		_ => return Err(EncodeError::WrongShape { actual: value, expected: type_id }),
	};

	let variant_type = ty.variants.iter().find(|v| v.name == variant.name);

	let variant_type = match variant_type {
		None => return Err(EncodeError::VariantNotFound { actual: variant, expected: type_id }),
		Some(v) => v,
	};

	variant_type.index.encode_to(bytes);
	encode_composite_fields(variant.values, &variant_type.fields, type_id, types, bytes)
}

fn encode_composite_fields<T>(
	composite: Composite<T>,
	fields: &[Field<PortableForm>],
	type_id: TypeId,
	types: &PortableRegistry,
	bytes: &mut Vec<u8>,
) -> Result<(), EncodeError<T>> {
	if fields.len() != composite.len() {
		return Err(EncodeError::CompositeIsWrongLength {
			actual: composite,
			expected: type_id,
			expected_len: fields.len(),
		})
	}

	// 0 length? Nothing more to do!
	if composite.is_empty() {
		return Ok(())
	}

	// Does the type we're encoding to have named fields or not?
	let is_named = fields[0].name.is_some();

	match (composite, is_named) {
		(Composite::Named(mut values), true) => {
			// Match up named values with those of the type we're encoding to.
			for field in fields.iter() {
				let field_name = field.name.as_ref().expect("field should be named; checked above");
				let value = values
					.iter()
					.position(|(n, _)| field_name == n)
					.map(|idx| values.swap_remove(idx).1);

				match value {
					Some(value) => {
						encode_value_as_type(value, field.ty, types, bytes)?;
					},
					None =>
						return Err(EncodeError::CompositeFieldIsMissing {
							expected: type_id,
							missing_field_name: field_name.clone(),
						}),
				}
			}

			Ok(())
		},
		(Composite::Unnamed(values), false) => {
			// Expect values in correct order only and encode.
			for (field, value) in fields.iter().zip(values) {
				encode_value_as_type(value, field.ty, types, bytes)?;
			}
			Ok(())
		},
		(values, _) => {
			// We expect named/unnamed fields and need the opposite.
			Err(EncodeError::CompositeIsWrongShape { actual: values, expected: type_id })
		},
	}
}

// Attempt to convert a given primitive value into the integer type
// required, failing with an appropriate EncodeValueError if not successful.
macro_rules! primitive_to_integer {
	($id:ident, $prim:ident, $context:expr => $ty:ident) => {{
		macro_rules! err {
			() => {
				EncodeError::WrongShape {
					actual: Value { context: $context, value: ValueDef::Primitive($prim) },
					expected: $id,
				}
			};
		}
		let out: Result<$ty, _> = match $prim {
			Primitive::U128(v) => v.try_into().map_err(|_| err!()),
			Primitive::I128(v) => v.try_into().map_err(|_| err!()),
			// Treat chars as u32s to mirror what we do for decoding:
			Primitive::Char(v) => (v as u32).try_into().map_err(|_| err!()),
			_ => Err(err!()),
		};
		out
	}};
}

fn encode_primitive_value<T>(
	value: Value<T>,
	type_id: TypeId,
	ty: &TypeDefPrimitive,
	bytes: &mut Vec<u8>,
) -> Result<(), EncodeError<T>> {
	let primitive = match value.value {
		ValueDef::Primitive(primitive) => primitive,
		_ => return Err(EncodeError::WrongShape { actual: value, expected: type_id }),
	};

	// Attempt to encode our value type into the expected shape.
	match (ty, primitive) {
		(TypeDefPrimitive::Bool, Primitive::Bool(bool)) => {
			bool.encode_to(bytes);
		},
		(TypeDefPrimitive::Char, Primitive::Char(c)) => {
			// Treat chars as u32's
			(c as u32).encode_to(bytes);
		},
		(TypeDefPrimitive::Str, Primitive::String(s)) => {
			s.encode_to(bytes);
		},
		(TypeDefPrimitive::I256, Primitive::I256(a)) => {
			a.encode_to(bytes);
		},
		(TypeDefPrimitive::U256, Primitive::U256(a)) => {
			a.encode_to(bytes);
		},
		(TypeDefPrimitive::U8, primitive) => {
			primitive_to_integer!(type_id, primitive, value.context => u8)?.encode_to(bytes);
		},
		(TypeDefPrimitive::U16, primitive) => {
			primitive_to_integer!(type_id, primitive, value.context => u16)?.encode_to(bytes);
		},
		(TypeDefPrimitive::U32, primitive) => {
			primitive_to_integer!(type_id, primitive, value.context => u32)?.encode_to(bytes);
		},
		(TypeDefPrimitive::U64, primitive) => {
			primitive_to_integer!(type_id, primitive, value.context => u64)?.encode_to(bytes);
		},
		(TypeDefPrimitive::U128, primitive) => {
			primitive_to_integer!(type_id, primitive, value.context => u128)?.encode_to(bytes);
		},
		(TypeDefPrimitive::I8, primitive) => {
			primitive_to_integer!(type_id, primitive, value.context => i8)?.encode_to(bytes);
		},
		(TypeDefPrimitive::I16, primitive) => {
			primitive_to_integer!(type_id, primitive, value.context => i16)?.encode_to(bytes);
		},
		(TypeDefPrimitive::I32, primitive) => {
			primitive_to_integer!(type_id, primitive, value.context => i32)?.encode_to(bytes);
		},
		(TypeDefPrimitive::I64, primitive) => {
			primitive_to_integer!(type_id, primitive, value.context => i64)?.encode_to(bytes);
		},
		(TypeDefPrimitive::I128, primitive) => {
			primitive_to_integer!(type_id, primitive, value.context => i128)?.encode_to(bytes);
		},
		(_, primitive) => {
			return Err(EncodeError::WrongShape {
				// Reconstruct a Value to give back:
				actual: Value { context: value.context, value: ValueDef::Primitive(primitive) },
				expected: type_id,
			})
		},
	}
	Ok(())
}

fn encode_compact_value<T>(
	value: Value<T>,
	type_id: TypeId,
	ty: &TypeDefCompact<PortableForm>,
	types: &PortableRegistry,
	bytes: &mut Vec<u8>,
) -> Result<(), EncodeError<T>> {
	// Types that are compact encodable:
	enum CompactTy {
		U8,
		U16,
		U32,
		U64,
		U128,
	}

	// Resolve to a primitive type inside the compact encoded type (or fail if
	// we hit some type we wouldn't know how to work with).
	let mut inner_ty_id = ty.type_param.id;
	let inner_ty = loop {
		let inner_ty = types
			.resolve(inner_ty_id)
			.ok_or_else(|| EncodeError::TypeIdNotFound(inner_ty_id.into()))?
			.type_def
			.clone();

		match inner_ty {
			TypeDef::Composite(c) =>
				if c.fields.len() == 1 {
					inner_ty_id = c.fields[0].ty.id;
				} else {
					return Err(EncodeError::CannotCompactEncode(inner_ty_id.into()))
				},
			TypeDef::Tuple(t) =>
				if t.fields.len() == 1 {
					inner_ty_id = t.fields[0].id;
				} else {
					return Err(EncodeError::CannotCompactEncode(inner_ty_id.into()))
				},
			TypeDef::Primitive(primitive) => {
				break match primitive {
					// These are the primitives that we can compact encode:
					TypeDefPrimitive::U8 => CompactTy::U8,
					TypeDefPrimitive::U16 => CompactTy::U16,
					TypeDefPrimitive::U32 => CompactTy::U32,
					TypeDefPrimitive::U64 => CompactTy::U64,
					TypeDefPrimitive::U128 => CompactTy::U128,
					_ => return Err(EncodeError::CannotCompactEncode(inner_ty_id.into())),
				}
			},
			TypeDef::Variant(_)
			| TypeDef::Sequence(_)
			| TypeDef::Array(_)
			| TypeDef::Compact(_)
			| TypeDef::BitSequence(_) => return Err(EncodeError::CannotCompactEncode(inner_ty_id.into())),
		}
	};

	// resolve to the innermost value that we have in the same way, expecting to get out
	// a single primitive value.
	let mut value = value;
	let inner_primitive = {
		loop {
			match value.value {
				ValueDef::Composite(c) =>
					if c.len() == 1 {
						value = c.into_values().next().expect("length of 1; value should exist");
					} else {
						return Err(EncodeError::WrongShape {
							actual: Value { context: value.context, value: ValueDef::Composite(c) },
							expected: inner_ty_id.into(),
						})
					},
				ValueDef::Primitive(primitive) => break primitive,
				ValueDef::Variant(_) | ValueDef::BitSequence(_) =>
					return Err(EncodeError::WrongShape {
						actual: value,
						expected: inner_ty_id.into(),
					}),
			}
		}
	};

	// Try to compact encode the primitive type we have into the type asked for:
	match inner_ty {
		CompactTy::U8 => {
			let val = primitive_to_integer!(type_id, inner_primitive, value.context => u8)?;
			Compact(val).encode_to(bytes);
		},
		CompactTy::U16 => {
			let val = primitive_to_integer!(type_id, inner_primitive, value.context => u16)?;
			Compact(val).encode_to(bytes);
		},
		CompactTy::U32 => {
			let val = primitive_to_integer!(type_id, inner_primitive, value.context => u32)?;
			Compact(val).encode_to(bytes);
		},
		CompactTy::U64 => {
			let val = primitive_to_integer!(type_id, inner_primitive, value.context => u64)?;
			Compact(val).encode_to(bytes);
		},
		CompactTy::U128 => {
			let val = primitive_to_integer!(type_id, inner_primitive, value.context => u128)?;
			Compact(val).encode_to(bytes);
		},
	};

	Ok(())
}
