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

use crate::scale_decode::visitor::TypeId;

/// This represents a compact encoded type.
pub struct Compact<'info, 'c, T> {
	val: T,
	locations: &'c [CompactLocation<'info>],
}

impl<'info, 'c, T: Copy> Compact<'info, 'c, T> {
	pub(crate) fn new(val: T, locations: &'c [CompactLocation<'info>]) -> Compact<'info, 'c, T> {
		Compact { val, locations }
	}
	/// Return the value that was compact-encoded.
	pub fn value(&self) -> T {
		self.val
	}
	/// Compact values can be nested inside named or unnamed fields in structs.
	/// This provides back a slice of these locations, in case such nesting matters.
	pub fn locations(&self) -> &'c [CompactLocation<'info>] {
		self.locations
	}
}

/// A pointer to what the compact value is contained within.
#[derive(Clone, Copy, Debug)]
pub enum CompactLocation<'info> {
	/// We're in an unnamed composite (struct) with the type ID given.
	UnnamedComposite(TypeId),
	/// We're in a named composite (struct) with the type ID given, and the compact
	/// value lives inside the field with the given name.
	NamedComposite(TypeId, &'info str),
	/// We're at a primitive type with the type ID given; the compact value itself.
	Primitive(TypeId),
}

impl<'info> CompactLocation<'info> {
	/// Return the Primitive type of this location, if one exists.
	pub fn as_primitive(self) -> Option<TypeId> {
		match self {
			CompactLocation::Primitive(t) => Some(t),
			_ => None,
		}
	}
}

// Default values for locations are never handed back, but they are
// stored on the StackArray in the "unused" positions. We could avoid needing
// this with some unsafe code.
impl<'info> Default for CompactLocation<'info> {
	fn default() -> Self {
		CompactLocation::Primitive(TypeId::default())
	}
}
