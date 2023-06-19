// This file was taken from scale-decode (Parity Technologies (UK))
// https://github.com/paritytech/scale-decode/
// And was adapted by Supercomputing Systems AG.
//
// Copyright (C) 2022-2023 Parity Technologies (UK) Ltd. (admin@parity.io)
// This file is a part of the scale-decode crate.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//         http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Based on https://github.com/paritytech/scale-decode/tree/5a982798a7987f2b79858f726478a1c415e1b416

use alloc::vec::Vec;

mod impls;

pub mod error;
pub mod visitor;

pub use error::Error;
pub use visitor::{DecodeError, IgnoreVisitor, Visitor};

// Used in trait definitions.
pub use scale_info::PortableRegistry;

/// This trait is implemented for any type `T` where `T` implements [`IntoVisitor`] and the errors returned
/// from this [`Visitor`] can be converted into [`Error`]. It's essentially a convenience wrapper around
/// [`visitor::decode_with_visitor`] that mirrors `scale-encode`'s `EncodeAsType`.
pub trait DecodeAsType: Sized {
	/// Given some input bytes, a `type_id`, and type registry, attempt to decode said bytes into
	/// `Self`. Implementations should modify the `&mut` reference to the bytes such that any bytes
	/// not used in the course of decoding are still pointed to after decoding is complete.
	fn decode_as_type(
		input: &mut &[u8],
		type_id: u32,
		types: &PortableRegistry,
	) -> Result<Self, Error>;
}

impl<T> DecodeAsType for T
where
	T: IntoVisitor,
	Error: From<<T::Visitor as Visitor>::Error>,
{
	fn decode_as_type(
		input: &mut &[u8],
		type_id: u32,
		types: &scale_info::PortableRegistry,
	) -> Result<Self, Error> {
		let res = visitor::decode_with_visitor(input, type_id, types, T::into_visitor())?;
		Ok(res)
	}
}

/// This is similar to [`DecodeAsType`], except that it's instead implemented for types that can be given a list of
/// fields denoting the type being decoded from and attempt to do this decoding. This is generally implemented just
/// for tuple and struct types, and is automatically implemented via the [`macro@DecodeAsType`] macro.
pub trait DecodeAsFields: Sized {
	/// Given some bytes and some fields denoting their structure, attempt to decode.
	fn decode_as_fields<'info>(
		input: &mut &[u8],
		fields: &mut dyn FieldIter<'info>,
		types: &'info PortableRegistry,
	) -> Result<Self, Error>;
}

/// A representation of a single field to be encoded via [`DecodeAsFields::decode_as_fields`].
#[derive(Debug, Clone, Copy)]
pub struct Field<'a> {
	name: Option<&'a str>,
	id: u32,
}

impl<'a> Field<'a> {
	/// Construct a new field with an ID and optional name.
	pub fn new(id: u32, name: Option<&'a str>) -> Self {
		Field { id, name }
	}
	/// Create a new unnamed field.
	pub fn unnamed(id: u32) -> Self {
		Field { name: None, id }
	}
	/// Create a new named field.
	pub fn named(id: u32, name: &'a str) -> Self {
		Field { name: Some(name), id }
	}
	/// The field name, if any.
	pub fn name(&self) -> Option<&'a str> {
		self.name
	}
	/// The field ID.
	pub fn id(&self) -> u32 {
		self.id
	}
}

/// An iterator over a set of fields.
pub trait FieldIter<'a>: Iterator<Item = Field<'a>> {}
impl<'a, T> FieldIter<'a> for T where T: Iterator<Item = Field<'a>> {}

/// This trait can be implemented on any type that has an associated [`Visitor`] responsible for decoding
/// SCALE encoded bytes to it. If you implement this on some type and the [`Visitor`] that you return has
/// an error type that converts into [`Error`], then you'll also get a [`DecodeAsType`] implementation for free.
pub trait IntoVisitor {
	/// The visitor type used to decode SCALE encoded bytes to `Self`.
	type Visitor: for<'scale, 'info> visitor::Visitor<Value<'scale, 'info> = Self>;
	/// A means of obtaining this visitor.
	fn into_visitor() -> Self::Visitor;
}

// In a few places, we need an empty path with a lifetime that outlives 'info,
// so here's one that lives forever that we can use.
#[doc(hidden)]
pub static EMPTY_SCALE_INFO_PATH: &scale_info::Path<scale_info::form::PortableForm> =
	&scale_info::Path { segments: Vec::new() };
