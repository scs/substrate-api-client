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

//! This module provides a [`Context`] type, which tracks the path
//! that we're attempting to encode to aid in error reporting.

use alloc::{borrow::Cow, vec::Vec};

/// A cheaply clonable opaque context which allows us to track the current
/// location into a type that we're trying to encode, to aid in
/// error reporting.
#[derive(Clone, Default, Debug)]
pub struct Context {
	path: Vec<Location>,
}

impl Context {
	/// Construct a new, empty context.
	pub fn new() -> Context {
		Default::default()
	}
	/// Return a new context with the given location appended.
	pub fn push(&mut self, loc: Location) {
		self.path.push(loc);
	}
	/// Return the current path.
	pub fn path(&self) -> Path<'_> {
		Path(Cow::Borrowed(&self.path))
	}
}

/// The current path that we're trying to encode.
pub struct Path<'a>(Cow<'a, Vec<Location>>);

impl<'a> Path<'a> {
	/// Cheaply convert the path to an owned version.
	pub fn into_owned(self) -> Path<'static> {
		Path(Cow::Owned(self.0.into_owned()))
	}
	/// Return each location visited, oldest first
	pub fn locations(&self) -> impl Iterator<Item = &Location> {
		self.0.iter()
	}
}

impl<'a> core::fmt::Display for Path<'a> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		for (idx, loc) in self.0.iter().enumerate() {
			if idx != 0 {
				f.write_str(".")?;
			}
			match &loc.inner {
				Loc::Field(name) => f.write_str(name)?,
				Loc::Index(i) => write!(f, "[{i}]")?,
				Loc::Variant(name) => write!(f, "({name})")?,
			}
		}
		Ok(())
	}
}

/// Some location, like a field, variant or index in an array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
	inner: Loc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Loc {
	Field(Cow<'static, str>),
	Index(usize),
	Variant(Cow<'static, str>),
}

impl Location {
	/// This represents some struct field.
	pub fn field(name: impl Into<Cow<'static, str>>) -> Self {
		Location { inner: Loc::Field(name.into()) }
	}
	/// This represents some variant name.
	pub fn variant(name: impl Into<Cow<'static, str>>) -> Self {
		Location { inner: Loc::Variant(name.into()) }
	}
	/// This represents a tuple or array index.
	pub fn idx(i: usize) -> Self {
		Location { inner: Loc::Index(i) }
	}
}
