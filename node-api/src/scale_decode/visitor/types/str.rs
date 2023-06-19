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

use crate::scale_decode::visitor::DecodeError;
use codec::{Compact, Decode};

/// This represents a string, but defers proper decoding of it until it's asked for,
/// and avoids allocating.
pub struct Str<'scale> {
	len: usize,
	compact_len: usize,
	bytes: &'scale [u8],
}

impl<'scale> Str<'scale> {
	pub(crate) fn new(bytes: &'scale [u8]) -> Result<Str<'scale>, DecodeError> {
		// Strings are just encoded the same as bytes; a length prefix and then
		// the raw bytes. decode the length but keep all of the bytes that represent this
		// encoded string (and the rest of the input) around so that we can provide a
		// consistent interface with Array/Sequence etc.
		let remaining_bytes = &mut &*bytes;
		let len = <Compact<u64>>::decode(remaining_bytes)?.0 as usize;
		let compact_len = bytes.len() - remaining_bytes.len();

		Ok(Str { len, bytes, compact_len })
	}
	/// The length of the string.
	pub fn len(&self) -> usize {
		self.len
	}
	/// The bytes left in the input, starting from this string.
	pub fn bytes_from_start(&self) -> &'scale [u8] {
		self.bytes
	}
	/// The bytes remaining in the input after this string.
	pub fn bytes_after(&self) -> &'scale [u8] {
		&self.bytes[self.compact_len + self.len..]
	}
	/// Is the string zero bytes long?
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}
	/// return a string, failing if the bytes could not be properly utf8-decoded.
	pub fn as_str(&self) -> Result<&'scale str, DecodeError> {
		let start = self.compact_len;
		let end = start + self.len;
		alloc::str::from_utf8(&self.bytes[start..end]).map_err(DecodeError::InvalidStr)
	}
}
