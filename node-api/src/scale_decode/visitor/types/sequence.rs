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

use super::array::{Array, ArrayItem};
use crate::scale_decode::{
	visitor::{DecodeError, Visitor},
	DecodeAsType,
};
use codec::{Compact, Decode};
use scale_info::PortableRegistry;

/// This enables a visitor to decode items from a sequence type.
pub struct Sequence<'scale, 'info> {
	bytes: &'scale [u8],
	// Mostly we just delegate to our Array logic for working with sequences.
	// The only thing we need to do otherwise is decode the compact encoded
	// length from the beginning and keep track of the bytes including that.
	values: Array<'scale, 'info>,
}

impl<'scale, 'info> Sequence<'scale, 'info> {
	pub(crate) fn new(
		bytes: &'scale [u8],
		type_id: u32,
		types: &'info PortableRegistry,
	) -> Result<Sequence<'scale, 'info>, DecodeError> {
		// Sequences are prefixed with their length in bytes. Make a note of this,
		// as well as the number of bytes
		let item_bytes = &mut &*bytes;
		let len = <Compact<u64>>::decode(item_bytes)?.0 as usize;

		Ok(Sequence { bytes, values: Array::new(item_bytes, type_id, len, types) })
	}
	/// Skip over all bytes associated with this sequence. After calling this,
	/// [`Self::bytes_from_undecoded()`] will represent the bytes after this sequence.
	pub fn skip_decoding(&mut self) -> Result<(), DecodeError> {
		self.values.skip_decoding()
	}
	/// The bytes representing this sequence and anything following it.
	pub fn bytes_from_start(&self) -> &'scale [u8] {
		self.bytes
	}
	/// The bytes that have not yet been decoded in this sequence (this never includes the
	/// compact length preceeding the sequence items) and anything following it.
	pub fn bytes_from_undecoded(&self) -> &'scale [u8] {
		self.values.bytes_from_undecoded()
	}
	/// The number of un-decoded items remaining in this sequence.
	pub fn remaining(&self) -> usize {
		self.values.remaining()
	}
	/// Decode an item from the sequence by providing a visitor to handle it.
	pub fn decode_item<V: Visitor>(
		&mut self,
		visitor: V,
	) -> Option<Result<V::Value<'scale, 'info>, V::Error>> {
		self.values.decode_item(visitor)
	}
}

// Iterating returns a representation of each field in the tuple type.
impl<'scale, 'info> Iterator for Sequence<'scale, 'info> {
	type Item = Result<SequenceItem<'scale, 'info>, DecodeError>;
	fn next(&mut self) -> Option<Self::Item> {
		Some(self.values.next()?.map(|item| SequenceItem { item }))
	}
}

/// A single item in the Sequence.
#[derive(Copy, Clone)]
pub struct SequenceItem<'scale, 'info> {
	// Same implementation under the hood as ArrayItem:
	item: ArrayItem<'scale, 'info>,
}

impl<'scale, 'info> SequenceItem<'scale, 'info> {
	/// The bytes associated with this item.
	pub fn bytes(&self) -> &'scale [u8] {
		self.item.bytes()
	}
	/// The type ID associated with this item.
	pub fn type_id(&self) -> u32 {
		self.item.type_id()
	}
	/// Decode this item using a visitor.
	pub fn decode_with_visitor<V: Visitor>(
		&self,
		visitor: V,
	) -> Result<V::Value<'scale, 'info>, V::Error> {
		self.item.decode_with_visitor(visitor)
	}
	/// Decode this item into a specific type via [`DecodeAsType`].
	pub fn decode_as_type<T: DecodeAsType>(&self) -> Result<T, crate::scale_decode::Error> {
		self.item.decode_as_type()
	}
}

impl<'scale, 'info> crate::scale_decode::visitor::DecodeItemIterator<'scale, 'info>
	for Sequence<'scale, 'info>
{
	fn decode_item<'a, V: Visitor>(
		&mut self,
		visitor: V,
	) -> Option<Result<V::Value<'scale, 'info>, V::Error>> {
		self.decode_item(visitor)
	}
}
