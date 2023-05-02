/*
   Copyright 2019 Supercomputing Systems AG

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

	   http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.

*/

//! Re-definition of substrate trait
//! Needed to become independent of sp_runtime

use alloc::vec::Vec;
use codec::Encode;
use sp_runtime::traits::Extrinsic;

/// This represents the hasher used by a node to hash things like block headers
/// and extrinsics.
// https://github.com/paritytech/subxt/blob/ce0a82e3227efb0eae131f025da5f839d9623e15/subxt/src/config/mod.rs#L71
pub trait Hasher {
	/// The type given back from the hash operation
	type Output;

	/// Hash some bytes to the given output type.
	fn hash(s: &[u8]) -> Self::Output;

	/// Hash some SCALE encodable type to the given output type.
	fn hash_of<S: Encode>(s: &S) -> Self::Output {
		let out = s.encode();
		Self::hash(&out)
	}
}
/// This represents the block header type used by a node.
// https://github.com/paritytech/subxt/blob/ce0a82e3227efb0eae131f025da5f839d9623e15/subxt/src/config/mod.rs#L85
pub trait Header: Sized + Encode {
	/// The block number type for this header.
	type Number: Into<u64>;
	/// The hasher used to hash this header.
	type Hasher: Hasher; //Header::Hash; type Hashing: Hash<Output = Self::Hash>;

	/// Return the block number of this header.
	fn number(&self) -> Self::Number;

	/// Hash this header.
	fn hash(&self) -> <Self::Hasher as Hasher>::Output {
		Self::Hasher::hash_of(self)
	}
}

/// This represents the block header type used by a node.
// https://github.com/paritytech/substrate/blob/master/primitives/runtime/src/traits.rs
pub trait Block: Sized + Encode {
	/// Type for extrinsics.
	type Extrinsic: Extrinsic + Encode;
	/// Header type.
	type Header: Header<Hasher = Self::Hasher>;
	/// Block hash type.
	type Hasher: Hasher;

	/// Returns a reference to the header.
	fn header(&self) -> &Self::Header;
	/// Returns a reference to the list of extrinsics.
	fn extrinsics(&self) -> &[Self::Extrinsic];
	/// Split the block into header and list of extrinsics.
	fn deconstruct(self) -> (Self::Header, Vec<Self::Extrinsic>);
	/// Creates new block from header and extrinsics.
	fn new(header: Self::Header, extrinsics: Vec<Self::Extrinsic>) -> Self;
	/// Returns the hash of the block.
	fn hash(&self) -> <Self::Hasher as Hasher>::Output {
		<<Self::Header as Header>::Hasher>::hash_of(self.header())
	}
	/// Creates an encoded block from the given `header` and `extrinsics` without requiring the
	/// creation of an instance.
	fn encode_from(header: &Self::Header, extrinsics: &[Self::Extrinsic]) -> Vec<u8>;
}
