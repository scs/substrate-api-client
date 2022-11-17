// This file was taken from scale-value (Parity Technologies (UK))
// https://github.com/paritytech/scale-value/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

use crate::alloc::{string::String, vec::Vec};
use bitvec::{order::Lsb0, vec::BitVec};
use core::{convert::From, fmt::Debug};
use either::Either;

/// [`Value`] holds a representation of some value that has been decoded, as well as some arbitrary context.
///
/// Not all SCALE encoded types have an similar-named value; for instance, the values corresponding to
/// sequence, array and composite types can all be represented with [`Composite`]. Only enough information
/// is preserved here to to be able to encode and decode SCALE bytes with a known type to and from [`Value`]s
/// losslessly.
#[derive(Debug, Clone, PartialEq)]
pub struct Value<T = ()> {
	/// The shape and associated data for this Value
	pub value: ValueDef<T>,
	/// Some additional arbitrary context that can be associated with a value.
	pub context: T,
}

impl Value<()> {
	/// Create a new named composite value without additional context.
	pub fn named_composite(values: Vec<(String, Value<()>)>) -> Value<()> {
		Value { value: ValueDef::Composite(Composite::Named(values)), context: () }
	}
	/// Create a new unnamed composite value without additional context.
	pub fn unnamed_composite(values: Vec<Value<()>>) -> Value<()> {
		Value { value: ValueDef::Composite(Composite::Unnamed(values)), context: () }
	}
	/// Create a new variant value without additional context.
	pub fn variant<S: Into<String>>(name: S, values: Composite<()>) -> Value<()> {
		Value { value: ValueDef::Variant(Variant { name: name.into(), values }), context: () }
	}
	/// Create a new variant value with named fields and without additional context.
	pub fn named_variant<S: Into<String>>(name: S, fields: Vec<(String, Value<()>)>) -> Value<()> {
		Value { value: ValueDef::Variant(Variant::named_fields(name, fields)), context: () }
	}
	/// Create a new variant value with tuple-like fields and without additional context.
	pub fn unnamed_variant<S: Into<String>>(name: S, fields: Vec<Value<()>>) -> Value<()> {
		Value { value: ValueDef::Variant(Variant::unnamed_fields(name, fields)), context: () }
	}
	/// Create a new bit sequence value without additional context.
	pub fn bit_sequence(bitseq: BitSequence) -> Value<()> {
		Value { value: ValueDef::BitSequence(bitseq), context: () }
	}
	/// Create a new primitive value without additional context.
	pub fn primitive(primitive: Primitive) -> Value<()> {
		Value { value: ValueDef::Primitive(primitive), context: () }
	}
	/// Create a new string value without additional context.
	pub fn string<S: Into<String>>(val: S) -> Value<()> {
		Value { value: ValueDef::Primitive(Primitive::String(val.into())), context: () }
	}
	/// Create a new boolean value without additional context.
	pub fn bool(val: bool) -> Value<()> {
		Value { value: ValueDef::Primitive(Primitive::Bool(val)), context: () }
	}
	/// Create a new char without additional context.
	pub fn char(val: char) -> Value<()> {
		Value { value: ValueDef::Primitive(Primitive::Char(val)), context: () }
	}
	/// Create a new unsigned integer without additional context.
	pub fn uint<N: Into<u128>>(val: N) -> Value<()> {
		Value { value: ValueDef::Primitive(Primitive::uint(val)), context: () }
	}
	/// Create a new signed integer without additional context.
	pub fn int<N: Into<i128>>(val: N) -> Value<()> {
		Value { value: ValueDef::Primitive(Primitive::int(val)), context: () }
	}
}

impl Value<()> {
	/// Create a new value with no associated context.
	pub fn without_context(value: ValueDef<()>) -> Value<()> {
		Value { value, context: () }
	}
}

impl<T> Value<T> {
	/// Create a new value with some associated context.
	pub fn with_context(value: ValueDef<T>, context: T) -> Value<T> {
		Value { value, context }
	}
	/// Remove the context.
	pub fn remove_context(self) -> Value<()> {
		self.map_context(|_| ())
	}
	/// Map the context to some different type.
	pub fn map_context<F, U>(self, mut f: F) -> Value<U>
	where
		F: Clone + FnMut(T) -> U,
	{
		Value { context: f(self.context), value: self.value.map_context(f) }
	}
}

/// The underlying shape of a given value.
#[derive(Clone, PartialEq)]
pub enum ValueDef<T> {
	/// A named or unnamed struct-like, array-like or tuple-like set of values.
	Composite(Composite<T>),
	/// An enum variant.
	Variant(Variant<T>),
	/// A sequence of bits (which is more compactly encoded using [`bitvec`])
	BitSequence(BitSequence),
	/// Any of the primitive values we can have.
	Primitive(Primitive),
}

impl<T> ValueDef<T> {
	/// Map the context to some different type.
	pub fn map_context<F, U>(self, f: F) -> ValueDef<U>
	where
		F: Clone + FnMut(T) -> U,
	{
		match self {
			ValueDef::Composite(val) => ValueDef::Composite(val.map_context(f)),
			ValueDef::Variant(val) => ValueDef::Variant(val.map_context(f)),
			ValueDef::BitSequence(val) => ValueDef::BitSequence(val),
			ValueDef::Primitive(val) => ValueDef::Primitive(val),
		}
	}
}

impl<T: Debug> Debug for ValueDef<T> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::Composite(val) => Debug::fmt(val, f),
			Self::Variant(val) => Debug::fmt(val, f),
			Self::Primitive(val) => Debug::fmt(val, f),
			Self::BitSequence(val) => Debug::fmt(val, f),
		}
	}
}

impl<T> From<BitSequence> for ValueDef<T> {
	fn from(val: BitSequence) -> Self {
		ValueDef::BitSequence(val)
	}
}

impl From<BitSequence> for Value<()> {
	fn from(val: BitSequence) -> Self {
		Value::without_context(val.into())
	}
}

/// A named or unnamed struct-like, array-like or tuple-like set of values.
/// This is used to represent a range of composite values on their own, or
/// as values for a specific [`Variant`].
#[derive(Clone, Debug, PartialEq)]
pub enum Composite<T> {
	/// Eg `{ foo: 2, bar: false }`
	Named(Vec<(String, Value<T>)>),
	/// Eg `(2, false)`
	Unnamed(Vec<Value<T>>),
}

impl<T> Composite<T> {
	/// Return the number of values stored in this composite type.
	pub fn len(&self) -> usize {
		match self {
			Composite::Named(values) => values.len(),
			Composite::Unnamed(values) => values.len(),
		}
	}

	/// Is the composite type empty?
	pub fn is_empty(&self) -> bool {
		match self {
			Composite::Named(values) => values.is_empty(),
			Composite::Unnamed(values) => values.is_empty(),
		}
	}

	/// Iterate over the values stored in this composite type.
	pub fn into_values(self) -> impl Iterator<Item = Value<T>> {
		match self {
			Composite::Named(values) => Either::Left(values.into_iter().map(|(_k, v)| v)),
			Composite::Unnamed(values) => Either::Right(values.into_iter()),
		}
	}

	/// Map the context to some different type.
	pub fn map_context<F, U>(self, f: F) -> Composite<U>
	where
		F: Clone + FnMut(T) -> U,
	{
		match self {
			Composite::Named(values) => {
				// Note: Optimally I'd pass `&mut f` into each iteration to avoid cloning,
				// but this leads to a type recusion error because F becomes `&mut F`, which can
				// (at type level) recurse here again and become `&mut &mut F` and so on. Since
				// that's no good; just require `Clone` to avoid altering the type.
				let vals =
					values.into_iter().map(move |(k, v)| (k, v.map_context(f.clone()))).collect();
				Composite::Named(vals)
			},
			Composite::Unnamed(values) => {
				let vals = values.into_iter().map(move |v| v.map_context(f.clone())).collect();
				Composite::Unnamed(vals)
			},
		}
	}
}

impl<V: Into<Value<()>>> From<Vec<V>> for Composite<()> {
	fn from(vals: Vec<V>) -> Self {
		let vals = vals.into_iter().map(|v| v.into()).collect();
		Composite::Unnamed(vals)
	}
}

impl<V: Into<Value<()>>> From<Vec<V>> for ValueDef<()> {
	fn from(vals: Vec<V>) -> Self {
		ValueDef::Composite(vals.into())
	}
}

impl<V: Into<Value<()>>> From<Vec<V>> for Value<()> {
	fn from(vals: Vec<V>) -> Self {
		Value::without_context(vals.into())
	}
}

impl<K: Into<String>, V: Into<Value<()>>> From<Vec<(K, V)>> for Composite<()> {
	fn from(vals: Vec<(K, V)>) -> Self {
		let vals = vals.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
		Composite::Named(vals)
	}
}

impl<K: Into<String>, V: Into<Value<()>>> From<Vec<(K, V)>> for ValueDef<()> {
	fn from(vals: Vec<(K, V)>) -> Self {
		ValueDef::Composite(vals.into())
	}
}

impl<K: Into<String>, V: Into<Value<()>>> From<Vec<(K, V)>> for Value<()> {
	fn from(vals: Vec<(K, V)>) -> Self {
		Value::without_context(vals.into())
	}
}

impl<T> From<Composite<T>> for ValueDef<T> {
	fn from(val: Composite<T>) -> Self {
		ValueDef::Composite(val)
	}
}

/// This represents the value of a specific variant from an enum, and contains
/// the name of the variant, and the named/unnamed values associated with it.
#[derive(Clone, Debug, PartialEq)]
pub struct Variant<T> {
	/// The name of the variant.
	pub name: String,
	/// Values for each of the named or unnamed fields associated with this variant.
	pub values: Composite<T>,
}

impl<T> Variant<T> {
	/// Construct a variant with named fields.
	pub fn named_fields<S: Into<String>>(name: S, fields: Vec<(String, Value<T>)>) -> Variant<T> {
		Variant { name: name.into(), values: Composite::Named(fields) }
	}
	/// Construct a variant with tuple-like fields.
	pub fn unnamed_fields<S: Into<String>>(name: S, fields: Vec<Value<T>>) -> Variant<T> {
		Variant { name: name.into(), values: Composite::Unnamed(fields) }
	}
	/// Map the context to some different type.
	pub fn map_context<F, U>(self, f: F) -> Variant<U>
	where
		F: Clone + FnMut(T) -> U,
	{
		Variant { name: self.name, values: self.values.map_context(f) }
	}
}

impl<T> From<Variant<T>> for ValueDef<T> {
	fn from(val: Variant<T>) -> Self {
		ValueDef::Variant(val)
	}
}

/// A "primitive" value (this includes strings).
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
	/// A boolean value.
	Bool(bool),
	/// A single character.
	Char(char),
	/// A string.
	String(String),
	/// A u128 value.
	U128(u128),
	/// An i128 value.
	I128(i128),
	/// An unsigned 256 bit number (internally represented as a 32 byte array).
	U256([u8; 32]),
	/// A signed 256 bit number (internally represented as a 32 byte array).
	I256([u8; 32]),
}

impl Primitive {
	/// Create a new unsigned integer without additional context.
	pub fn uint<N: Into<u128>>(val: N) -> Primitive {
		Primitive::U128(val.into())
	}
	/// Create a new signed integer without additional context.
	pub fn int<N: Into<i128>>(val: N) -> Primitive {
		Primitive::I128(val.into())
	}
}

impl<T> From<Primitive> for ValueDef<T> {
	fn from(val: Primitive) -> Self {
		ValueDef::Primitive(val)
	}
}

macro_rules! impl_primitive_type {
    ($($variant:ident($ty:ty),)*) => {$(
        impl From<$ty> for Primitive {
            fn from(val: $ty) -> Self {
                Primitive::$variant(val)
            }
        }

        impl<T> From<$ty> for ValueDef<T> {
            fn from(val: $ty) -> Self {
                ValueDef::Primitive(val.into())
            }
        }

        impl From<$ty> for Value<()> {
            fn from(val: $ty) -> Self {
                Value::without_context(val.into())
            }
        }
    )*}
}

impl_primitive_type!(Bool(bool), Char(char), String(String), U128(u128), I128(i128),);

/// A sequence of bits.
pub type BitSequence = BitVec<u8, Lsb0>;
