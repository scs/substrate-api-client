// Copyright (C) 2023 Parity Technologies (UK) Ltd. (admin@parity.io)
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

use crate::scale_decode::{DecodeAsType, DecodeError, IgnoreVisitor, Visitor};
use scale_info::PortableRegistry;

/// This enables a visitor to decode items from an array type.
pub struct Array<'scale, 'info> {
	bytes: &'scale [u8],
	item_bytes: &'scale [u8],
	type_id: u32,
	types: &'info PortableRegistry,
	remaining: usize,
}

impl<'scale, 'info> Array<'scale, 'info> {
	pub(crate) fn new(
		bytes: &'scale [u8],
		type_id: u32,
		len: usize,
		types: &'info PortableRegistry,
	) -> Array<'scale, 'info> {
		Array { bytes, item_bytes: bytes, type_id, types, remaining: len }
	}
	/// Skip over all bytes associated with this array. After calling this,
	/// [`Self::bytes_from_undecoded()`] will represent the bytes after this array.
	pub fn skip_decoding(&mut self) -> Result<(), DecodeError> {
		while self.remaining > 0 {
			self.decode_item(IgnoreVisitor).transpose()?;
		}
		Ok(())
	}
	/// The bytes representing this array and anything following it.
	pub fn bytes_from_start(&self) -> &'scale [u8] {
		self.bytes
	}
	/// The bytes that have not yet been decoded in this array and anything following
	/// it.
	pub fn bytes_from_undecoded(&self) -> &'scale [u8] {
		self.item_bytes
	}
	/// The number of un-decoded items remaining in this array.
	pub fn remaining(&self) -> usize {
		self.remaining
	}
	/// Are there any un-decoded items remaining in this array.
	pub fn is_empty(&self) -> bool {
		self.remaining == 0
	}
	/// Decode an item from the array by providing a visitor to handle it.
	pub fn decode_item<V: Visitor>(
		&mut self,
		visitor: V,
	) -> Option<Result<V::Value<'scale, 'info>, V::Error>> {
		if self.remaining == 0 {
			return None
		}

		let b = &mut self.item_bytes;
		// Don't return here; decrement bytes and remaining properly first and then return, so that
		// calling decode_item again works as expected.
		let res =
			crate::scale_decode::visitor::decode_with_visitor(b, self.type_id, self.types, visitor);
		self.item_bytes = *b;
		self.remaining -= 1;
		Some(res)
	}
}

// Iterating returns a representation of each field in the tuple type.
impl<'scale, 'info> Iterator for Array<'scale, 'info> {
	type Item = Result<ArrayItem<'scale, 'info>, DecodeError>;
	fn next(&mut self) -> Option<Self::Item> {
		// Record details we need before we decode and skip over the thing:
		let num_bytes_before = self.item_bytes.len();
		let item_bytes = self.item_bytes;

		if let Err(e) = self.decode_item(IgnoreVisitor)? {
			return Some(Err(e))
		};

		// How many bytes did we skip over? What bytes represent the thing we decoded?
		let num_bytes_after = self.item_bytes.len();
		let res_bytes = &item_bytes[..num_bytes_before - num_bytes_after];

		Some(Ok(ArrayItem { bytes: res_bytes, type_id: self.type_id, types: self.types }))
	}
}

/// A single item in the array.
#[derive(Copy, Clone)]
pub struct ArrayItem<'scale, 'info> {
	bytes: &'scale [u8],
	type_id: u32,
	types: &'info PortableRegistry,
}

impl<'scale, 'info> ArrayItem<'scale, 'info> {
	/// The bytes associated with this item.
	pub fn bytes(&self) -> &'scale [u8] {
		self.bytes
	}
	/// The type ID associated with this item.
	pub fn type_id(&self) -> u32 {
		self.type_id
	}
	/// Decode this item using a visitor.
	pub fn decode_with_visitor<V: Visitor>(
		&self,
		visitor: V,
	) -> Result<V::Value<'scale, 'info>, V::Error> {
		crate::scale_decode::visitor::decode_with_visitor(
			&mut &*self.bytes,
			self.type_id,
			self.types,
			visitor,
		)
	}
	/// Decode this item into a specific type via [`DecodeAsType`].
	pub fn decode_as_type<T: DecodeAsType>(&self) -> Result<T, crate::scale_decode::Error> {
		T::decode_as_type(&mut &*self.bytes, self.type_id, self.types)
	}
}

impl<'scale, 'info> crate::scale_decode::visitor::DecodeItemIterator<'scale, 'info>
	for Array<'scale, 'info>
{
	fn decode_item<'a, V: Visitor>(
		&mut self,
		visitor: V,
	) -> Option<Result<V::Value<'scale, 'info>, V::Error>> {
		self.decode_item(visitor)
	}
}
