// Copyright 2019-2021 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! RPC parameters, orginally belonging to jsonrpsee:
//! https://github.com/paritytech/jsonrpsee
//! It is copied & pasted here to avoid std dependencies.

use serde::Serialize;

#[derive(Debug)]
pub struct RpcParams(ParamsBuilder);

impl RpcParams {
	/// Construct a new [`RpcParams`].
	pub fn new() -> Self {
		Self::default()
	}

	/// Insert a plain value into the builder.
	pub fn insert<P: Serialize>(&mut self, value: P) -> Result<(), serde_json::Error> {
		self.0.insert(value)
	}

	/// Finish the building process and return a JSON compatible string.
	pub fn build(self) -> Option<String> {
		self.0.build()
	}
}

impl Default for RpcParams {
	fn default() -> Self {
		Self(ParamsBuilder::positional())
	}
}
/// Initial number of bytes for a parameter length.
const PARAM_BYTES_CAPACITY: usize = 128;

/// Generic parameter builder that serializes parameters to bytes.
/// This produces a JSON compatible String.
///
/// The implementation relies on `Vec<u8>` to hold the serialized
/// parameters in memory for the following reasons:
///   1. Other serialization methods than `serde_json::to_writer` would internally
///      have an extra heap allocation for temporarily holding the value in memory.
///   2. `io::Write` is not implemented for `String` required for serialization.
#[derive(Debug)]
pub(crate) struct ParamsBuilder {
	bytes: Vec<u8>,
	start: char,
	end: char,
}

impl ParamsBuilder {
	/// Construct a new [`ParamsBuilder`] with custom start and end tokens.
	/// The inserted values are wrapped by the _start_ and _end_ tokens.
	fn new(start: char, end: char) -> Self {
		ParamsBuilder { bytes: Vec::new(), start, end }
	}

	/// Construct a new [`ParamsBuilder`] for positional parameters equivalent to a JSON array object.
	pub(crate) fn positional() -> Self {
		Self::new('[', ']')
	}

	#[allow(unused)]
	/// Construct a new [`ParamsBuilder`] for named parameters equivalent to a JSON map object.
	pub(crate) fn named() -> Self {
		Self::new('{', '}')
	}

	/// Initialize the internal vector if it is empty:
	///  - allocate [`PARAM_BYTES_CAPACITY`] to avoid resizing
	///  - add the `start` character.
	///
	/// # Note
	///
	/// Initialization is needed prior to inserting elements.
	fn maybe_initialize(&mut self) {
		if self.bytes.is_empty() {
			self.bytes.reserve(PARAM_BYTES_CAPACITY);
			self.bytes.push(self.start as u8);
		}
	}

	#[allow(unused)]
	/// Insert a named value (key, value) pair into the builder.
	/// The _name_ and _value_ are delimited by the `:` token.
	pub(crate) fn insert_named<P: Serialize>(
		&mut self,
		name: &str,
		value: P,
	) -> Result<(), serde_json::Error> {
		self.maybe_initialize();

		serde_json::to_writer(&mut self.bytes, name)?;
		self.bytes.push(b':');
		serde_json::to_writer(&mut self.bytes, &value)?;
		self.bytes.push(b',');

		Ok(())
	}

	/// Insert a plain value into the builder.
	pub(crate) fn insert<P: Serialize>(&mut self, value: P) -> Result<(), serde_json::Error> {
		self.maybe_initialize();

		serde_json::to_writer(&mut self.bytes, &value)?;
		self.bytes.push(b',');

		Ok(())
	}

	/// Finish the building process and return a JSON compatible string.
	pub(crate) fn build(mut self) -> Option<String> {
		if self.bytes.is_empty() {
			return None
		}

		let idx = self.bytes.len() - 1;
		if self.bytes[idx] == b',' {
			self.bytes[idx] = self.end as u8;
		} else {
			self.bytes.push(self.end as u8);
		}

		// Safety: This is safe because JSON does not emit invalid UTF-8.
		Some(unsafe { String::from_utf8_unchecked(self.bytes) })
	}
}
