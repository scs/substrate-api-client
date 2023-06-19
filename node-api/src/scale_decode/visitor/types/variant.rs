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

use crate::scale_decode::{
	visitor::{Composite, DecodeError},
	Field,
};
use scale_info::{form::PortableForm, Path, PortableRegistry, TypeDefVariant};

/// A representation of the a variant type.
pub struct Variant<'scale, 'info> {
	bytes: &'scale [u8],
	variant: &'info scale_info::Variant<PortableForm>,
	fields: Composite<'scale, 'info>,
}

impl<'scale, 'info> Variant<'scale, 'info> {
	pub(crate) fn new(
		bytes: &'scale [u8],
		path: &'info Path<PortableForm>,
		ty: &'info TypeDefVariant<PortableForm>,
		types: &'info PortableRegistry,
	) -> Result<Variant<'scale, 'info>, DecodeError> {
		let index = *bytes.first().ok_or(DecodeError::NotEnoughInput)?;
		let item_bytes = &bytes[1..];

		// Does a variant exist with the index we're looking for?
		let variant = ty
			.variants
			.iter()
			.find(|v| v.index == index)
			.ok_or_else(|| DecodeError::VariantNotFound(index, ty.clone()))?;

		// Allow decoding of the fields:
		let mut fields_iter = variant.fields.iter().map(|f| Field::new(f.ty.id, f.name.as_deref()));
		let fields = Composite::new(item_bytes, path, &mut fields_iter, types);

		Ok(Variant { bytes, variant, fields })
	}
}

impl<'scale, 'info> Variant<'scale, 'info> {
	/// Skip over all bytes associated with this variant. After calling this,
	/// [`Self::bytes_from_undecoded()`] will represent the bytes after this variant.
	pub fn skip_decoding(&mut self) -> Result<(), DecodeError> {
		self.fields.skip_decoding()
	}
	/// The bytes representing this sequence and anything following it.
	pub fn bytes_from_start(&self) -> &'scale [u8] {
		self.bytes
	}
	/// The bytes that have not yet been decoded in this variant (this never includes the
	/// variant index at the front) and anything following it.
	pub fn bytes_from_undecoded(&self) -> &'scale [u8] {
		self.fields.bytes_from_undecoded()
	}
	/// Path to this type.
	pub fn path(&self) -> &'info Path<PortableForm> {
		self.fields.path()
	}
	/// The name of the variant.
	pub fn name(&self) -> &'info str {
		&self.variant.name
	}
	/// The index of the variant.
	pub fn index(&self) -> u8 {
		self.variant.index
	}
	/// Access the variant fields.
	pub fn fields(&mut self) -> &mut Composite<'scale, 'info> {
		&mut self.fields
	}
}
