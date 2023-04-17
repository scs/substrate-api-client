// This file was taken from scale-value (Parity Technologies (UK))
// https://github.com/paritytech/scale-value/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

use super::{
	bit_sequence::{get_bitsequence_details, BitOrderTy, BitSequenceError, BitStoreTy},
	value::{BitSequence, Composite, Primitive, Value, ValueDef, Variant},
	ScaleType as Type, ScaleTypeDef as TypeDef, TypeId,
};
use crate::alloc::{string::String, vec, vec::Vec};
use bitvec::{
	order::{BitOrder, Lsb0, Msb0},
	store::BitStore,
	vec::BitVec,
};
use codec::{Compact, Decode};
use scale_info::{
	form::PortableForm, Field, PortableRegistry, TypeDefArray, TypeDefBitSequence, TypeDefCompact,
	TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeDefVariant,
};

/// An error decoding SCALE bytes into a [`Value`].
#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
	/// Some error emitted from a [`codec::Decode`] impl.
	CodecError(codec::Error),
	/// We could not convert the [`u32`] that we found into a valid [`char`].
	InvalidChar(u32),
	/// We could not find the type given in the type registry provided.
	TypeIdNotFound(u32),
	/// We expected more bytes to finish decoding, but could not find them.
	Eof,
	/// We found a variant that does not match with any in the type we're trying to decode from.
	VariantNotFound(u8, scale_info::TypeDefVariant<PortableForm>),
	/// The type we're trying to decode is supposed to be compact encoded, but that is not possible.
	CannotDecodeCompactIntoType(Type),
	/// We ran into an error trying to decode a bit sequence.
	BitSequenceError(BitSequenceError),
}

impl From<codec::Error> for DecodeError {
	fn from(error: codec::Error) -> Self {
		DecodeError::CodecError(error)
	}
}

/// Decode data according to the [`TypeId`] provided.
/// The provided pointer to the data slice will be moved forwards as needed
/// depending on what was decoded.
pub fn decode_value_as_type<Id: Into<TypeId>>(
	data: &mut &[u8],
	ty_id: Id,
	types: &PortableRegistry,
) -> Result<Value<TypeId>, DecodeError> {
	let ty_id = ty_id.into();
	let ty = types
		.resolve(ty_id.id())
		.ok_or_else(|| DecodeError::TypeIdNotFound(ty_id.id()))?;

	let value = match &ty.type_def {
		TypeDef::Composite(inner) =>
			decode_composite_value(data, inner, types).map(ValueDef::Composite),
		TypeDef::Sequence(inner) =>
			decode_sequence_value(data, inner, types).map(ValueDef::Composite),
		TypeDef::Array(inner) => decode_array_value(data, inner, types).map(ValueDef::Composite),
		TypeDef::Tuple(inner) => decode_tuple_value(data, inner, types).map(ValueDef::Composite),
		TypeDef::Variant(inner) => decode_variant_value(data, inner, types).map(ValueDef::Variant),
		TypeDef::Primitive(inner) => decode_primitive_value(data, inner).map(ValueDef::Primitive),
		TypeDef::Compact(inner) => decode_compact_value(data, inner, types),
		TypeDef::BitSequence(inner) =>
			decode_bit_sequence_value(data, inner, types).map(ValueDef::BitSequence),
	}?;

	Ok(Value { value, context: ty_id })
}

fn decode_composite_value(
	data: &mut &[u8],
	ty: &TypeDefComposite<PortableForm>,
	types: &PortableRegistry,
) -> Result<Composite<TypeId>, DecodeError> {
	decode_fields(data, &ty.fields, types)
}

fn decode_variant_value(
	data: &mut &[u8],
	ty: &TypeDefVariant<PortableForm>,
	types: &PortableRegistry,
) -> Result<Variant<TypeId>, DecodeError> {
	let index = *data.first().ok_or(DecodeError::Eof)?;
	*data = &data[1..];

	// Does a variant exist with the index we're looking for?
	let variant = ty
		.variants
		.iter()
		.find(|v| v.index == index)
		.ok_or_else(|| DecodeError::VariantNotFound(index, ty.clone()))?;

	let fields = decode_fields(data, &variant.fields, types)?;
	Ok(Variant { name: variant.name.clone(), values: fields })
}

/// Variant and Composite types both have fields; this will decode them into values.
fn decode_fields(
	data: &mut &[u8],
	fields: &[Field<PortableForm>],
	types: &PortableRegistry,
) -> Result<Composite<TypeId>, DecodeError> {
	let are_named = fields.iter().any(|f| f.name.is_some());
	let named_field_vals = fields.iter().map(|f| {
		let name = f.name.as_ref().cloned().unwrap_or_default();
		decode_value_as_type(data, f.ty, types).map(|val| (name, val))
	});

	if are_named {
		let vals = named_field_vals.collect::<Result<_, _>>()?;
		Ok(Composite::Named(vals))
	} else {
		let vals = named_field_vals.map(|r| r.map(|(_, v)| v)).collect::<Result<_, _>>()?;
		Ok(Composite::Unnamed(vals))
	}
}

fn decode_sequence_value(
	data: &mut &[u8],
	ty: &TypeDefSequence<PortableForm>,
	types: &PortableRegistry,
) -> Result<Composite<TypeId>, DecodeError> {
	// We assume that the sequence is preceeded by a compact encoded length, so that
	// we know how many values to try pulling out of the data.
	let len = Compact::<u64>::decode(data)?;
	let values: Vec<_> = (0..len.0)
		.map(|_| decode_value_as_type(data, ty.type_param, types))
		.collect::<Result<_, _>>()?;

	Ok(Composite::Unnamed(values))
}

fn decode_array_value(
	data: &mut &[u8],
	ty: &TypeDefArray<PortableForm>,
	types: &PortableRegistry,
) -> Result<Composite<TypeId>, DecodeError> {
	// The length is known based on the type we want to decode into, so we pull out the number of items according
	// to that, and don't need a length to exist in the SCALE encoded bytes
	let values: Vec<_> = (0..ty.len)
		.map(|_| decode_value_as_type(data, ty.type_param, types))
		.collect::<Result<_, _>>()?;

	Ok(Composite::Unnamed(values))
}

fn decode_tuple_value(
	data: &mut &[u8],
	ty: &TypeDefTuple<PortableForm>,
	types: &PortableRegistry,
) -> Result<Composite<TypeId>, DecodeError> {
	let values: Vec<_> = ty
		.fields
		.iter()
		.map(|f| decode_value_as_type(data, f, types))
		.collect::<Result<_, _>>()?;

	Ok(Composite::Unnamed(values))
}

fn decode_primitive_value(
	data: &mut &[u8],
	ty: &TypeDefPrimitive,
) -> Result<Primitive, DecodeError> {
	let val = match ty {
		TypeDefPrimitive::Bool => Primitive::Bool(bool::decode(data)?),
		TypeDefPrimitive::Char => {
			// Treat chars as u32's
			let val = u32::decode(data)?;
			Primitive::Char(char::from_u32(val).ok_or(DecodeError::InvalidChar(val))?)
		},
		TypeDefPrimitive::Str => Primitive::String(String::decode(data)?),
		TypeDefPrimitive::U8 => Primitive::uint(u8::decode(data)?),
		TypeDefPrimitive::U16 => Primitive::uint(u16::decode(data)?),
		TypeDefPrimitive::U32 => Primitive::uint(u32::decode(data)?),
		TypeDefPrimitive::U64 => Primitive::uint(u64::decode(data)?),
		TypeDefPrimitive::U128 => Primitive::uint(u128::decode(data)?),
		TypeDefPrimitive::U256 => Primitive::U256(<[u8; 32]>::decode(data)?),
		TypeDefPrimitive::I8 => Primitive::int(i8::decode(data)?),
		TypeDefPrimitive::I16 => Primitive::int(i16::decode(data)?),
		TypeDefPrimitive::I32 => Primitive::int(i32::decode(data)?),
		TypeDefPrimitive::I64 => Primitive::int(i64::decode(data)?),
		TypeDefPrimitive::I128 => Primitive::int(i128::decode(data)?),
		TypeDefPrimitive::I256 => Primitive::I256(<[u8; 32]>::decode(data)?),
	};
	Ok(val)
}

fn decode_compact_value(
	data: &mut &[u8],
	ty: &TypeDefCompact<PortableForm>,
	types: &PortableRegistry,
) -> Result<ValueDef<TypeId>, DecodeError> {
	fn decode_compact(
		data: &mut &[u8],
		inner: &Type,
		types: &PortableRegistry,
	) -> Result<ValueDef<TypeId>, DecodeError> {
		use TypeDefPrimitive::*;
		let val = match &inner.type_def {
			// It's obvious how to decode basic primitive unsigned types, since we have impls for them.
			TypeDef::Primitive(U8) =>
				ValueDef::Primitive(Primitive::uint(Compact::<u8>::decode(data)?.0)),
			TypeDef::Primitive(U16) =>
				ValueDef::Primitive(Primitive::uint(Compact::<u16>::decode(data)?.0)),
			TypeDef::Primitive(U32) =>
				ValueDef::Primitive(Primitive::uint(Compact::<u32>::decode(data)?.0)),
			TypeDef::Primitive(U64) =>
				ValueDef::Primitive(Primitive::uint(Compact::<u64>::decode(data)?.0)),
			TypeDef::Primitive(U128) =>
				ValueDef::Primitive(Primitive::uint(Compact::<u128>::decode(data)?.0)),
			// A struct with exactly 1 field containing one of the above types can be sensibly compact encoded/decoded.
			TypeDef::Composite(composite) => {
				if composite.fields.len() != 1 {
					return Err(DecodeError::CannotDecodeCompactIntoType(inner.clone()))
				}

				// What type is the 1 field that we are able to decode?
				let field = &composite.fields[0];
				let field_type_id = field.ty.id;
				let inner_ty = types
					.resolve(field_type_id)
					.ok_or(DecodeError::TypeIdNotFound(field_type_id))?;

				// Decode this inner type via compact decoding. This can recurse, in case
				// the inner type is also a 1-field composite type.
				let inner_value = Value {
					value: decode_compact(data, inner_ty, types)?,
					context: field.ty.into(),
				};

				// Wrap the inner type in a representation of this outer composite type.
				let composite = match &field.name {
					Some(name) => Composite::Named(vec![(name.clone(), inner_value)]),
					None => Composite::Unnamed(vec![inner_value]),
				};

				ValueDef::Composite(composite)
			},
			// For now, we give up if we have been asked for any other type:
			_cannot_decode_from =>
				return Err(DecodeError::CannotDecodeCompactIntoType(inner.clone())),
		};

		Ok(val)
	}

	// Pluck the inner type out and run it through our compact decoding logic.
	let inner = types
		.resolve(ty.type_param.id)
		.ok_or_else(|| DecodeError::TypeIdNotFound(ty.type_param.id))?;
	decode_compact(data, inner, types)
}

fn decode_bit_sequence_value(
	data: &mut &[u8],
	ty: &TypeDefBitSequence<PortableForm>,
	types: &PortableRegistry,
) -> Result<BitSequence, DecodeError> {
	let details = get_bitsequence_details(ty, types).map_err(DecodeError::BitSequenceError)?;

	fn to_bit_sequence<S: BitStore, O: BitOrder>(bits: BitVec<S, O>) -> BitSequence {
		bits.iter().by_vals().collect()
	}

	// Decode the native BitSequence type easily, or else convert to it from the type given.
	let bits = match details {
		(BitOrderTy::U8, BitStoreTy::Lsb0) => BitVec::<u8, Lsb0>::decode(data)?,
		(BitOrderTy::U8, BitStoreTy::Msb0) => to_bit_sequence(BitVec::<u8, Msb0>::decode(data)?),
		(BitOrderTy::U16, BitStoreTy::Lsb0) => to_bit_sequence(BitVec::<u16, Lsb0>::decode(data)?),
		(BitOrderTy::U16, BitStoreTy::Msb0) => to_bit_sequence(BitVec::<u16, Msb0>::decode(data)?),
		(BitOrderTy::U32, BitStoreTy::Lsb0) => to_bit_sequence(BitVec::<u32, Lsb0>::decode(data)?),
		(BitOrderTy::U32, BitStoreTy::Msb0) => to_bit_sequence(BitVec::<u32, Msb0>::decode(data)?),
		#[cfg(target_pointer_width = "64")]
		(BitOrderTy::U64, BitStoreTy::Lsb0) => to_bit_sequence(BitVec::<u64, Lsb0>::decode(data)?),
		#[cfg(target_pointer_width = "64")]
		(BitOrderTy::U64, BitStoreTy::Msb0) => to_bit_sequence(BitVec::<u64, Msb0>::decode(data)?),
	};

	Ok(bits)
}

#[cfg(test)]
mod test {

	use super::*;
	use codec::Encode;

	/// Given a type definition, return the PortableType and PortableRegistry
	/// that our decode functions expect.
	fn make_type<T: scale_info::TypeInfo + 'static>() -> (TypeId, PortableRegistry) {
		let m = scale_info::MetaType::new::<T>();
		let mut types = scale_info::Registry::new();
		let id = types.register_type(&m);
		let portable_registry: PortableRegistry = types.into();

		(id.into(), portable_registry)
	}

	/// Given a value to encode, and a representation of the decoded value, check that our decode functions
	/// successfully decodes the type to the expected value, based on the implicit SCALE type info that the type
	/// carries
	fn encode_decode_check<T: Encode + scale_info::TypeInfo + 'static>(val: T, exp: Value<()>) {
		encode_decode_check_explicit_info::<T, _>(val, exp)
	}

	/// Given a value to encode, a type to decode it back into, and a representation of
	/// the decoded value, check that our decode functions successfully decodes as expected.
	fn encode_decode_check_explicit_info<Ty: scale_info::TypeInfo + 'static, T: Encode>(
		val: T,
		ex: Value<()>,
	) {
		let encoded = val.encode();
		let encoded = &mut &*encoded;

		let (id, portable_registry) = make_type::<Ty>();

		// Can we decode?
		let val = decode_value_as_type(encoded, id, &portable_registry).expect("decoding failed");
		// Is the decoded value what we expected?
		assert_eq!(val.remove_context(), ex, "decoded value does not look like what we expected");
		// Did decoding consume all of the encoded bytes, as expected?
		assert_eq!(encoded.len(), 0, "decoding did not consume all of the encoded bytes");
	}

	#[test]
	fn decode_primitives() {
		encode_decode_check(true, Value::bool(true));
		encode_decode_check(false, Value::bool(false));
		encode_decode_check_explicit_info::<char, _>('a' as u32, Value::char('a'));
		encode_decode_check("hello", Value::string("hello"));
		encode_decode_check(
			"hello".to_string(), // String or &str (above) decode OK
			Value::string("hello"),
		);
		encode_decode_check(123u8, Value::uint(123u8));
		encode_decode_check(123u16, Value::uint(123u8));
		encode_decode_check(123u32, Value::uint(123u8));
		encode_decode_check(123u64, Value::uint(123u8));
		encode_decode_check(123u128, Value::uint(123u8));
		//// Todo [jsdw]: Can we test this if we need a TypeInfo param?:
		// encode_decode_check_explicit_info(
		// 	[123u8; 32], // Anything 32 bytes long will do here
		// 	Value::u256([123u8; 32]),
		// );
		encode_decode_check(123i8, Value::int(123i16));
		encode_decode_check(123i16, Value::int(123i16));
		encode_decode_check(123i32, Value::int(123i16));
		encode_decode_check(123i64, Value::int(123i16));
		encode_decode_check(123i128, Value::int(123i16));
		//// Todo [jsdw]: Can we test this if we need a TypeInfo param?:
		// encode_decode_check_explicit_info(
		// 	[123u8; 32], // Anything 32 bytes long will do here
		// 	Value::i256([123u8; 32]),
		// );
	}

	#[test]
	fn decode_compact_primitives() {
		encode_decode_check(Compact(123u8), Value::uint(123u8));
		encode_decode_check(Compact(123u16), Value::uint(123u8));
		encode_decode_check(Compact(123u32), Value::uint(123u8));
		encode_decode_check(Compact(123u64), Value::uint(123u8));
		encode_decode_check(Compact(123u128), Value::uint(123u8));
	}

	#[test]
	fn decode_compact_named_wrapper_struct() {
		// A struct that can be compact encoded:
		#[derive(Encode, scale_info::TypeInfo)]
		struct MyWrapper {
			inner: u32,
		}
		impl From<Compact<MyWrapper>> for MyWrapper {
			fn from(val: Compact<MyWrapper>) -> MyWrapper {
				val.0
			}
		}
		impl codec::CompactAs for MyWrapper {
			type As = u32;

			fn encode_as(&self) -> &Self::As {
				&self.inner
			}
			fn decode_from(inner: Self::As) -> Result<Self, codec::Error> {
				Ok(MyWrapper { inner })
			}
		}

		encode_decode_check(
			Compact(MyWrapper { inner: 123 }),
			Value::named_composite(vec![("inner".to_string(), Value::uint(123u8))]),
		);
	}

	#[test]
	fn decode_compact_unnamed_wrapper_struct() {
		// A struct that can be compact encoded:
		#[derive(Encode, scale_info::TypeInfo)]
		struct MyWrapper(u32);
		impl From<Compact<MyWrapper>> for MyWrapper {
			fn from(val: Compact<MyWrapper>) -> MyWrapper {
				val.0
			}
		}
		impl codec::CompactAs for MyWrapper {
			type As = u32;

			// Node the requirement to return something with a lifetime tied
			// to self here. This means that we can't implement this for things
			// more complex than wrapper structs (eg `Foo(u32,u32,u32,u32)`) without
			// shenanigans, meaning that (hopefully) supporting wrapper struct
			// decoding and nothing fancier is sufficient.
			fn encode_as(&self) -> &Self::As {
				&self.0
			}
			fn decode_from(inner: Self::As) -> Result<Self, codec::Error> {
				Ok(MyWrapper(inner))
			}
		}

		encode_decode_check(
			Compact(MyWrapper(123)),
			Value::unnamed_composite(vec![Value::uint(123u8)]),
		);
	}

	#[test]
	fn decode_sequence_array_tuple_types() {
		encode_decode_check(
			vec![1i32, 2, 3],
			Value::unnamed_composite(vec![Value::int(1), Value::int(2), Value::int(3)]),
		);
		encode_decode_check(
			[1i32, 2, 3], // compile-time length known
			Value::unnamed_composite(vec![Value::int(1), Value::int(2), Value::int(3)]),
		);
		encode_decode_check(
			(1i32, true, 123456u128),
			Value::unnamed_composite(vec![
				Value::int(1),
				Value::bool(true),
				Value::uint(123456u32),
			]),
		);
	}

	#[test]
	fn decode_variant_types() {
		#[derive(Encode, scale_info::TypeInfo)]
		enum MyEnum {
			Foo(bool),
			Bar { hi: String, other: u128 },
		}

		encode_decode_check(
			MyEnum::Foo(true),
			Value::unnamed_variant("Foo", vec![Value::bool(true)]),
		);
		encode_decode_check(
			MyEnum::Bar { hi: "hello".to_string(), other: 123 },
			Value::named_variant(
				"Bar",
				vec![
					("hi".to_string(), Value::string("hello".to_string())),
					("other".to_string(), Value::uint(123u8)),
				],
			),
		);
	}

	#[test]
	fn decode_composite_types() {
		#[derive(Encode, scale_info::TypeInfo)]
		struct Unnamed(bool, String, Vec<u8>);

		#[derive(Encode, scale_info::TypeInfo)]
		struct Named {
			is_valid: bool,
			name: String,
			bytes: Vec<u8>,
		}

		encode_decode_check(
			Unnamed(true, "James".into(), vec![1, 2, 3]),
			Value::unnamed_composite(vec![
				Value::bool(true),
				Value::string("James".to_string()),
				Value::unnamed_composite(vec![
					Value::uint(1u8),
					Value::uint(2u8),
					Value::uint(3u8),
				]),
			]),
		);
		encode_decode_check(
			Named { is_valid: true, name: "James".into(), bytes: vec![1, 2, 3] },
			Value::named_composite(vec![
				("is_valid".into(), Value::bool(true)),
				("name".into(), Value::string("James".to_string())),
				(
					"bytes".into(),
					Value::unnamed_composite(vec![
						Value::uint(1u8),
						Value::uint(2u8),
						Value::uint(3u8),
					]),
				),
			]),
		);
	}
}
