// This file was taken from scale-value (Parity Technologies (UK))
// https://github.com/paritytech/scale-value/
// And was adapted by Supercomputing Systems AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! Based on https://github.com/paritytech/scale-value/blob/430bfaf8f302dfcfc45d8d63c687628fd9b7fc25/src/scale_impls/decode.rs

use super::TypeId;
use crate::scale_value::{Composite, Primitive, Value, ValueDef, Variant};
use alloc::{borrow::ToOwned, vec::Vec};
use scale_decode::FieldIter;
use scale_info::{form::PortableForm, Path, PortableRegistry};

// This is emitted if something goes wrong decoding into a Value.
pub use scale_decode::visitor::DecodeError;

/// Decode data according to the [`TypeId`] provided.
/// The provided pointer to the data slice will be moved forwards as needed
/// depending on what was decoded.
pub fn decode_value_as_type(
	data: &mut &[u8],
	ty_id: TypeId,
	types: &PortableRegistry,
) -> Result<Value<TypeId>, DecodeError> {
	scale_decode::visitor::decode_with_visitor(data, ty_id, types, DecodeValueVisitor)
}

// Sequences, Tuples and Arrays all have the same methods, so decode them in the same way:
macro_rules! to_unnamed_composite {
	($value:ident, $type_id:ident) => {{
		let mut vals = Vec::with_capacity($value.remaining());
		while let Some(val) = $value.decode_item(DecodeValueVisitor) {
			let val = val?;
			vals.push(val);
		}
		Ok(Value { value: ValueDef::Composite(Composite::Unnamed(vals)), context: $type_id.0 })
	}};
}

// We can't implement this on `Value<TypeId>` because we have no TypeId to assign to the value.
impl scale_decode::DecodeAsFields for Composite<TypeId> {
	fn decode_as_fields<'info>(
		input: &mut &[u8],
		fields: &mut dyn FieldIter<'info>,
		types: &'info PortableRegistry,
	) -> Result<Self, scale_decode::Error> {
		// Build a Composite type to pass to a one-off visitor:
		static EMPTY_PATH: &Path<PortableForm> = &Path { segments: Vec::new() };
		let mut composite =
			scale_decode::visitor::types::Composite::new(input, EMPTY_PATH, fields, types);
		// Decode into a Composite value from this:
		let val = visit_composite(&mut composite);
		// Consume remaining bytes and update input cursor:
		composite.skip_decoding()?;
		*input = composite.bytes_from_undecoded();

		val.map_err(From::from)
	}
}

/// A [`scale_decode::Visitor`] implementation for decoding into [`Value`]s.
pub struct DecodeValueVisitor;

impl scale_decode::IntoVisitor for Value<TypeId> {
	type Visitor = DecodeValueVisitor;
	fn into_visitor() -> Self::Visitor {
		DecodeValueVisitor
	}
}

impl scale_decode::visitor::Visitor for DecodeValueVisitor {
	type Value<'scale, 'info> = Value<TypeId>;
	type Error = DecodeError;

	fn visit_bool<'scale, 'info>(
		self,
		value: bool,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		Ok(Value::bool(value).map_context(|_| type_id.0))
	}
	fn visit_char<'scale, 'info>(
		self,
		value: char,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		Ok(Value::char(value).map_context(|_| type_id.0))
	}
	fn visit_u8<'scale, 'info>(
		self,
		value: u8,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		self.visit_u128(value as u128, type_id)
	}
	fn visit_u16<'scale, 'info>(
		self,
		value: u16,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		self.visit_u128(value as u128, type_id)
	}
	fn visit_u32<'scale, 'info>(
		self,
		value: u32,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		self.visit_u128(value as u128, type_id)
	}
	fn visit_u64<'scale, 'info>(
		self,
		value: u64,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		self.visit_u128(value as u128, type_id)
	}
	fn visit_u128<'scale, 'info>(
		self,
		value: u128,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		Ok(Value::u128(value).map_context(|_| type_id.0))
	}
	fn visit_u256<'info>(
		self,
		value: &'_ [u8; 32],
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'_, 'info>, Self::Error> {
		Ok(Value { value: ValueDef::Primitive(Primitive::U256(*value)), context: type_id.0 })
	}
	fn visit_i8<'scale, 'info>(
		self,
		value: i8,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		self.visit_i128(value as i128, type_id)
	}
	fn visit_i16<'scale, 'info>(
		self,
		value: i16,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		self.visit_i128(value as i128, type_id)
	}
	fn visit_i32<'scale, 'info>(
		self,
		value: i32,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		self.visit_i128(value as i128, type_id)
	}
	fn visit_i64<'scale, 'info>(
		self,
		value: i64,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		self.visit_i128(value as i128, type_id)
	}
	fn visit_i128<'scale, 'info>(
		self,
		value: i128,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		Ok(Value::i128(value).map_context(|_| type_id.0))
	}
	fn visit_i256<'info>(
		self,
		value: &'_ [u8; 32],
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'_, 'info>, Self::Error> {
		Ok(Value { value: ValueDef::Primitive(Primitive::U256(*value)), context: type_id.0 })
	}
	fn visit_sequence<'scale, 'info>(
		self,
		value: &mut scale_decode::visitor::types::Sequence<'scale, 'info>,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		to_unnamed_composite!(value, type_id)
	}
	fn visit_tuple<'scale, 'info>(
		self,
		value: &mut scale_decode::visitor::types::Tuple<'scale, 'info>,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		to_unnamed_composite!(value, type_id)
	}
	fn visit_array<'scale, 'info>(
		self,
		value: &mut scale_decode::visitor::types::Array<'scale, 'info>,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		to_unnamed_composite!(value, type_id)
	}
	fn visit_bitsequence<'scale, 'info>(
		self,
		value: &mut scale_decode::visitor::types::BitSequence<'scale>,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		let bits: Result<_, _> = value.decode()?.collect();
		Ok(Value { value: ValueDef::BitSequence(bits?), context: type_id.0 })
	}
	fn visit_str<'scale, 'info>(
		self,
		value: &mut scale_decode::visitor::types::Str<'scale>,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		Ok(Value::string(value.as_str()?).map_context(|_| type_id.0))
	}
	fn visit_variant<'scale, 'info>(
		self,
		value: &mut scale_decode::visitor::types::Variant<'scale, 'info>,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		let values = visit_composite(value.fields())?;
		Ok(Value {
			value: ValueDef::Variant(Variant { name: value.name().to_owned(), values }),
			context: type_id.0,
		})
	}
	fn visit_composite<'scale, 'info>(
		self,
		value: &mut scale_decode::visitor::types::Composite<'scale, 'info>,
		type_id: scale_decode::visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		Ok(Value { value: ValueDef::Composite(visit_composite(value)?), context: type_id.0 })
	}
}

/// Extract a named/unnamed Composite type out of scale_decode's Composite.
fn visit_composite(
	value: &mut scale_decode::visitor::types::Composite<'_, '_>,
) -> Result<Composite<TypeId>, DecodeError> {
	let len = value.remaining();
	// if no fields, we'll always assume unnamed.
	let named = len > 0 && !value.has_unnamed_fields();

	if named {
		let mut vals = Vec::with_capacity(len);
		let mut name = value.peek_name();
		while let Some(v) = value.decode_item(DecodeValueVisitor) {
			let v = v?;
			vals.push((name.expect("all fields should be named; we have checked").to_owned(), v));
			// get the next field name now we've decoded one.
			name = value.peek_name();
		}
		Ok(Composite::Named(vals))
	} else {
		let mut vals = Vec::with_capacity(len);
		while let Some(v) = value.decode_item(DecodeValueVisitor) {
			let v = v?;
			vals.push(v);
		}
		Ok(Composite::Unnamed(vals))
	}
}

#[cfg(test)]
mod test {

	use super::*;
	use codec::{Compact, Encode};

	/// Given a type definition, return the PortableType and PortableRegistry
	/// that our decode functions expect.
	fn make_type<T: scale_info::TypeInfo + 'static>() -> (TypeId, PortableRegistry) {
		let m = scale_info::MetaType::new::<T>();
		let mut types = scale_info::Registry::new();
		let id = types.register_type(&m);
		let portable_registry: PortableRegistry = types.into();

		(id.id, portable_registry)
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
		encode_decode_check(123u8, Value::u128(123));
		encode_decode_check(123u16, Value::u128(123));
		encode_decode_check(123u32, Value::u128(123));
		encode_decode_check(123u64, Value::u128(123));
		encode_decode_check(123u128, Value::u128(123));
		//// Todo [jsdw]: Can we test this if we need a TypeInfo param?:
		// encode_decode_check_explicit_info(
		//     [123u8; 32], // Anything 32 bytes long will do here
		//     Value::u256([123u8; 32]),
		// );
		encode_decode_check(123i8, Value::i128(123));
		encode_decode_check(123i16, Value::i128(123));
		encode_decode_check(123i32, Value::i128(123));
		encode_decode_check(123i64, Value::i128(123));
		encode_decode_check(123i128, Value::i128(123));
		//// Todo [jsdw]: Can we test this if we need a TypeInfo param?:
		// encode_decode_check_explicit_info(
		//     [123u8; 32], // Anything 32 bytes long will do here
		//     Value::i256([123u8; 32]),
		// );
	}

	#[test]
	fn decode_compact_primitives() {
		encode_decode_check(Compact(123u8), Value::u128(123));
		encode_decode_check(Compact(123u16), Value::u128(123));
		encode_decode_check(Compact(123u32), Value::u128(123));
		encode_decode_check(Compact(123u64), Value::u128(123));
		encode_decode_check(Compact(123u128), Value::u128(123));
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

		encode_decode_check(Compact(MyWrapper { inner: 123 }), Value::u128(123));
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

		encode_decode_check(Compact(MyWrapper(123)), Value::u128(123));
	}

	#[test]
	fn decode_sequence_array_tuple_types() {
		encode_decode_check(
			vec![1i32, 2, 3],
			Value::unnamed_composite(vec![Value::i128(1), Value::i128(2), Value::i128(3)]),
		);
		encode_decode_check(
			[1i32, 2, 3], // compile-time length known
			Value::unnamed_composite(vec![Value::i128(1), Value::i128(2), Value::i128(3)]),
		);
		encode_decode_check(
			(1i32, true, 123456u128),
			Value::unnamed_composite(vec![Value::i128(1), Value::bool(true), Value::u128(123456)]),
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
					("other".to_string(), Value::u128(123)),
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
				Value::unnamed_composite(vec![Value::u128(1), Value::u128(2), Value::u128(3)]),
			]),
		);
		encode_decode_check(
			Named { is_valid: true, name: "James".into(), bytes: vec![1, 2, 3] },
			Value::named_composite(vec![
				("is_valid", Value::bool(true)),
				("name", Value::string("James".to_string())),
				(
					"bytes",
					Value::unnamed_composite(vec![Value::u128(1), Value::u128(2), Value::u128(3)]),
				),
			]),
		);
	}

	#[test]
	fn decoding_zero_length_composites_always_unnamed() {
		// The scale-info repr is just a composite, so we don't really track
		// whether the thing was named or not. either Value will convert back ok anyway.
		#[derive(Encode, scale_info::TypeInfo)]
		struct Named {}
		#[derive(Encode, scale_info::TypeInfo)]
		struct Unnamed();

		encode_decode_check(Unnamed(), Value::unnamed_composite(vec![]));
		encode_decode_check(Named {}, Value::unnamed_composite(vec![]));
	}

	#[test]
	fn decode_bit_sequence() {
		use scale_bits::bits;

		// scale-decode already tests this more thoroughly:
		encode_decode_check(bits![0, 1, 1, 0, 1, 0], Value::bit_sequence(bits![0, 1, 1, 0, 1, 0]));
	}
}
