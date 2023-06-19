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

//! Types used in the [`super::Visitor`] trait definition.

mod array;
mod bit_sequence;
mod compact;
mod composite;
mod sequence;
mod str;
mod tuple;
mod variant;

pub use self::str::Str;
pub use array::Array;
pub use bit_sequence::BitSequence;
pub use compact::{Compact, CompactLocation};
pub use composite::Composite;
pub use sequence::Sequence;
pub use tuple::Tuple;
pub use variant::Variant;
