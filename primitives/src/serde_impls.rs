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

//! Re-defintion of substrate primitives that do not implement
//! (De)Serialization in no_std. They can be converted to
//! the original substrate types with From / Into.
//! This may be omitted, if substrate allows serde impls also in no_std: https://github.com/paritytech/substrate/issues/12994

use alloc::{string::String, vec::Vec};
use codec::{Decode, Encode, MaxEncodedLen};
use impl_serde::serialize::{from_hex, FromHexError};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;
use sp_runtime::Justification;
use sp_version::{ApiId, ApisVec};

/// Hex-serialized shim for `Vec<u8>`.
// https://github.com/paritytech/substrate/blob/5aaf5f42a7850f00b15a14f635b67061d831ac2d/primitives/core/src/lib.rs#L131
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct Bytes(#[serde(with = "impl_serde::serialize")] pub Vec<u8>);

impl From<Vec<u8>> for Bytes {
	fn from(s: Vec<u8>) -> Self {
		Bytes(s)
	}
}

impl core::str::FromStr for Bytes {
	type Err = FromHexError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		from_hex(s).map(Bytes)
	}
}

impl From<Bytes> for sp_core::Bytes {
	fn from(bytes: Bytes) -> Self {
		Self(bytes.0)
	}
}

impl From<sp_core::Bytes> for Bytes {
	fn from(bytes: sp_core::Bytes) -> Self {
		Self(bytes.0)
	}
}

/// Storage key.
// https://github.com/paritytech/substrate/blob/cd2fdcf85eb96c53ce2a5d418d4338eb92f5d4f5/primitives/storage/src/lib.rs#L41-L43
#[derive(
	PartialEq,
	Eq,
	RuntimeDebug,
	Serialize,
	Deserialize,
	Hash,
	PartialOrd,
	Ord,
	Clone,
	Encode,
	Decode,
)]
pub struct StorageKey(#[serde(with = "impl_serde::serialize")] pub Vec<u8>);

impl From<Vec<u8>> for StorageKey {
	fn from(s: Vec<u8>) -> Self {
		StorageKey(s)
	}
}

impl AsRef<[u8]> for StorageKey {
	fn as_ref(&self) -> &[u8] {
		self.0.as_ref()
	}
}

impl From<StorageKey> for sp_core::storage::StorageKey {
	fn from(storage_key: StorageKey) -> Self {
		Self(storage_key.0)
	}
}

impl From<sp_core::storage::StorageKey> for StorageKey {
	fn from(storage_key: sp_core::storage::StorageKey) -> Self {
		Self(storage_key.0)
	}
}

/// Storage data associated to a [`StorageKey`].
// https://github.com/paritytech/substrate/blob/cd2fdcf85eb96c53ce2a5d418d4338eb92f5d4f5/primitives/storage/src/lib.rs#L148-L150
#[derive(
	PartialEq,
	Eq,
	RuntimeDebug,
	Serialize,
	Deserialize,
	Hash,
	PartialOrd,
	Ord,
	Clone,
	Encode,
	Decode,
	Default,
)]
pub struct StorageData(#[serde(with = "impl_serde::serialize")] pub Vec<u8>);

impl From<StorageData> for sp_core::storage::StorageData {
	fn from(storage_data: StorageData) -> Self {
		Self(storage_data.0)
	}
}

impl From<sp_core::storage::StorageData> for StorageData {
	fn from(storage_data: sp_core::storage::StorageData) -> Self {
		Self(storage_data.0)
	}
}

/// Storage change set
#[derive(RuntimeDebug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StorageChangeSet<Hash> {
	/// Block hash
	pub block: Hash,
	/// A list of changes
	pub changes: Vec<(StorageKey, Option<StorageData>)>,
}

impl<Hash> From<StorageChangeSet<Hash>> for sp_core::storage::StorageChangeSet<Hash> {
	fn from(storage_change_set: StorageChangeSet<Hash>) -> Self {
		Self {
			block: storage_change_set.block,
			changes: storage_change_set
				.changes
				.iter()
				.map(|(key, maybe_data)| {
					(key.clone().into(), maybe_data.as_ref().map(|data| data.clone().into()))
				})
				.collect(),
		}
	}
}

impl<Hash> From<sp_core::storage::StorageChangeSet<Hash>> for StorageChangeSet<Hash> {
	fn from(storage_change_set: sp_core::storage::StorageChangeSet<Hash>) -> Self {
		Self {
			block: storage_change_set.block,
			changes: storage_change_set
				.changes
				.into_iter()
				.map(|(key, maybe_data)| {
					let key: StorageKey = key.into();
					let maybe_data: Option<StorageData> = maybe_data.map(|data| data.into());
					(key, maybe_data)
				})
				.collect(),
		}
	}
}

/// Runtime version.
/// This should not be thought of as classic Semver (major/minor/tiny).
/// This triplet have different semantics and mis-interpretation could cause problems.
/// In particular: bug fixes should result in an increment of `spec_version` and possibly
/// `authoring_version`, absolutely not `impl_version` since they change the semantics of the
/// runtime.
// https://github.com/paritytech/substrate/blob/1b3ddae9dec6e7653b5d6ef0179df1af831f46f0/primitives/version/src/lib.rs#L152-L215
// FIXME: For now RuntimeVersion conversion is not implemented to the substrate RuntimeVersion.
// It's a little more complicated because of the RuntimeString, which is different in no_std than in std mode.
#[derive(
	Clone, PartialEq, Eq, Default, sp_runtime::RuntimeDebug, TypeInfo, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeVersion {
	/// Identifies the different Substrate runtimes. There'll be at least polkadot and node.
	/// A different on-chain spec_name to that of the native runtime would normally result
	/// in node not attempting to sync or author blocks.
	pub spec_name: String,

	/// Name of the implementation of the spec. This is of little consequence for the node
	/// and serves only to differentiate code of different implementation teams. For this
	/// codebase, it will be parity-polkadot. If there were a non-Rust implementation of the
	/// Polkadot runtime (e.g. C++), then it would identify itself with an accordingly different
	/// `impl_name`.
	pub impl_name: String,

	/// `authoring_version` is the version of the authorship interface. An authoring node
	/// will not attempt to author blocks unless this is equal to its native runtime.
	pub authoring_version: u32,

	/// Version of the runtime specification. A full-node will not attempt to use its native
	/// runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	/// `spec_version` and `authoring_version` are the same between Wasm and native.
	pub spec_version: u32,

	/// Version of the implementation of the specification. Nodes are free to ignore this; it
	/// serves only as an indication that the code is different; as long as the other two versions
	/// are the same then while the actual code may be different, it is nonetheless required to
	/// do the same thing.
	/// Non-consensus-breaking optimizations are about the only changes that could be made which
	/// would result in only the `impl_version` changing.
	pub impl_version: u32,

	/// List of supported API "features" along with their versions.
	#[serde(
		serialize_with = "apis_serialize::serialize",
		deserialize_with = "apis_serialize::deserialize"
	)]
	pub apis: ApisVec,
	//pub apis: alloc::borrow::Cow<'static, [([u8; 8], u32)]>,
	/// All existing dispatches are fully compatible when this number doesn't change. If this
	/// number changes, then `spec_version` must change, also.
	///
	/// This number must change when an existing dispatchable (module ID, dispatch ID) is changed,
	/// either through an alteration in its user-level semantics, a parameter
	/// added/removed/changed, a dispatchable being removed, a module being removed, or a
	/// dispatchable/module changing its index.
	///
	/// It need *not* change when a new module is added or when a dispatchable is added.
	pub transaction_version: u32,

	/// Version of the state implementation used by this runtime.
	/// Use of an incorrect version is consensus breaking.
	pub state_version: u8,
}

/// Abstraction over a substrate block and justification.
// https://github.com/paritytech/substrate/blob/fafc8e0ba8c98bd22b47913ded414e74a0fcb67f/primitives/runtime/src/generic/block.rs
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SignedBlock<Block> {
	/// Full block.
	pub block: Block,
	/// Block justification.
	pub justifications: Option<Justifications>,
}

impl<Block> From<sp_runtime::generic::SignedBlock<Block>> for SignedBlock<Block> {
	fn from(signed_block: sp_runtime::generic::SignedBlock<Block>) -> Self {
		Self {
			block: signed_block.block,
			justifications: signed_block.justifications.map(|justifactions| justifactions.into()),
		}
	}
}

impl<Block> From<SignedBlock<Block>> for sp_runtime::generic::SignedBlock<Block> {
	fn from(signed_block: SignedBlock<Block>) -> Self {
		Self {
			block: signed_block.block,
			justifications: signed_block.justifications.map(|justifactions| justifactions.into()),
		}
	}
}

/// Collection of justifications for a given block, multiple justifications may
/// be provided by different consensus engines for the same block.
// https://github.com/paritytech/substrate/blob/fafc8e0ba8c98bd22b47913ded414e74a0fcb67f/primitives/runtime/src/lib.rs#L125-L127
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub struct Justifications(pub Vec<Justification>);

impl From<sp_runtime::Justifications> for Justifications {
	fn from(justifications: sp_runtime::Justifications) -> Self {
		let mut justification_vec = Vec::new();
		for justification in justifications.iter() {
			justification_vec.push(justification.clone());
		}
		Self(justification_vec)
	}
}

impl From<Justifications> for sp_runtime::Justifications {
	fn from(justifications: Justifications) -> Self {
		let first_justifaction = justifications.0[0].clone();
		let mut sp_runtime_justifications: sp_runtime::Justifications = first_justifaction.into();
		for justification in justifications.0 {
			sp_runtime_justifications.append(justification);
		}
		sp_runtime_justifications
	}
}

/// The old (deprecated) weight type.
// https://github.com/paritytech/substrate/blob/d0540a79967cb06cd7598a4965c7c06afc788b0c/primitives/weights/src/lib.rs#L78
#[derive(
	Decode,
	Encode,
	PartialEq,
	Eq,
	Clone,
	Copy,
	RuntimeDebug,
	Default,
	TypeInfo,
	Serialize,
	Deserialize,
)]
#[serde(transparent)]
pub struct OldWeight(pub u64);

/// New weight type.
// https://github.com/paritytech/substrate/blob/7bbfe737a180e548ace7e819099dcb62cf48fa11/primitives/weights/src/weight_v2.rs#L25-L36
#[derive(
	Encode,
	Decode,
	MaxEncodedLen,
	TypeInfo,
	Eq,
	PartialEq,
	Copy,
	Clone,
	RuntimeDebug,
	Default,
	Serialize,
	Deserialize,
)]
pub struct Weight {
	#[codec(compact)]
	/// The weight of computational time used based on some reference hardware.
	ref_time: u64,
	#[codec(compact)]
	/// The weight of storage space used by proof of validity.
	proof_size: u64,
}

impl Weight {
	/// Set the reference time part of the weight.
	pub const fn set_ref_time(mut self, c: u64) -> Self {
		self.ref_time = c;
		self
	}

	/// Set the storage size part of the weight.
	pub const fn set_proof_size(mut self, c: u64) -> Self {
		self.proof_size = c;
		self
	}

	/// Return the reference time part of the weight.
	pub const fn ref_time(&self) -> u64 {
		self.ref_time
	}

	/// Return the storage size part of the weight.
	pub const fn proof_size(&self) -> u64 {
		self.proof_size
	}

	/// Construct [`Weight`] from weight parts, namely reference time and proof size weights.
	pub const fn from_parts(ref_time: u64, proof_size: u64) -> Self {
		Self { ref_time, proof_size }
	}

	/// Construct [`Weight`] from the same weight for all parts.
	pub const fn from_all(value: u64) -> Self {
		Self { ref_time: value, proof_size: value }
	}

	/// Return a [`Weight`] where all fields are zero.
	pub const fn zero() -> Self {
		Self { ref_time: 0, proof_size: 0 }
	}
}

impl From<sp_weights::Weight> for Weight {
	fn from(weight: sp_weights::Weight) -> Self {
		Weight::from_parts(weight.ref_time(), weight.proof_size())
	}
}

impl From<Weight> for sp_weights::Weight {
	fn from(weight: Weight) -> Self {
		sp_weights::Weight::from_parts(weight.ref_time(), weight.proof_size())
	}
}

// Copied from sp_version (only available in std in the substrate version).
// https://github.com/paritytech/substrate/blob/1b3ddae9dec6e7653b5d6ef0179df1af831f46f0/primitives/version/src/lib.rs#L392-L393
mod apis_serialize {
	use super::*;
	use impl_serde::serialize as bytes;
	use serde::{de, ser::SerializeTuple, Serializer};

	#[derive(Serialize)]
	struct ApiId<'a>(#[serde(serialize_with = "serialize_bytesref")] &'a super::ApiId, &'a u32);

	pub fn serialize<S>(apis: &ApisVec, ser: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let len = apis.len();
		let mut seq = ser.serialize_tuple(len)?;
		for (api, ver) in &**apis {
			seq.serialize_element(&ApiId(api, ver))?;
		}
		seq.end()
	}

	pub fn serialize_bytesref<S>(&apis: &&super::ApiId, ser: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		bytes::serialize(apis, ser)
	}

	#[derive(Deserialize)]
	struct ApiIdOwned(#[serde(deserialize_with = "deserialize_bytes")] super::ApiId, u32);

	pub fn deserialize<'de, D>(deserializer: D) -> Result<ApisVec, D::Error>
	where
		D: de::Deserializer<'de>,
	{
		struct Visitor;
		impl<'de> de::Visitor<'de> for Visitor {
			type Value = ApisVec;

			fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
				formatter.write_str("a sequence of api id and version tuples")
			}

			fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
			where
				V: de::SeqAccess<'de>,
			{
				let mut apis = Vec::new();
				while let Some(value) = visitor.next_element::<ApiIdOwned>()? {
					apis.push((value.0, value.1));
				}
				Ok(apis.into())
			}
		}
		deserializer.deserialize_seq(Visitor)
	}

	pub fn deserialize_bytes<'de, D>(d: D) -> Result<super::ApiId, D::Error>
	where
		D: de::Deserializer<'de>,
	{
		let mut arr = [0; 8];
		bytes::deserialize_check_len(d, bytes::ExpectedLen::Exact(&mut arr[..]))?;
		Ok(arr)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use codec::Encode;
	use core::str::FromStr;
	use primitive_types::H256;

	#[test]
	fn from_substrate_bytes_to_bytes_works() {
		let string = "0x12341560";
		let bytes = Bytes::from_str(string).unwrap();
		let substrate_bytes: sp_core::Bytes = bytes.clone().into();
		let original_bytes: Bytes = substrate_bytes.into();

		assert_eq!(original_bytes, bytes);
	}

	#[test]
	fn from_substrate_storage_data_to_storage_data_works() {
		let test_vec = "test_string".encode();
		let storage_data = StorageData(test_vec);
		let substrate_storage_data: sp_core::storage::StorageData = storage_data.clone().into();
		let original_storage_data: StorageData = substrate_storage_data.into();

		assert_eq!(original_storage_data, storage_data);
	}

	#[test]
	fn from_substrate_storage_key_to_storage_key_works() {
		let test_vec = "test_string".encode();
		let storage_key = StorageKey(test_vec);
		let substrate_storage_key: sp_core::storage::StorageKey = storage_key.clone().into();
		let original_storage_key: StorageKey = substrate_storage_key.into();

		assert_eq!(original_storage_key, storage_key);
	}

	#[test]
	fn from_substrate_change_set_to_change_set_works() {
		let test_vec = "test_data".encode();
		let storage_data = StorageData(test_vec);
		let test_vec = "test_key".encode();
		let storage_key = StorageKey(test_vec);
		let changes = vec![(storage_key, Some(storage_data))];
		let hash = H256::random();

		let change_set = StorageChangeSet { block: hash, changes: changes.clone() };

		let substrate_change_set: sp_core::storage::StorageChangeSet<H256> = change_set.into();
		let original_change_set: StorageChangeSet<H256> = substrate_change_set.into();

		assert_eq!(original_change_set.block, hash);
		assert_eq!(original_change_set.changes, changes);
	}

	#[test]
	fn from_substrate_justifications_to_justification_works() {
		let encoded_justifcation = "test_string".encode();
		let consensus_engine_id: [u8; 4] = [1, 2, 3, 4];
		let justification: Justification = (consensus_engine_id, encoded_justifcation);
		let justifications = Justifications(vec![justification]);

		let substrate_justifications: sp_runtime::Justifications = justifications.clone().into();
		let original_justifications: Justifications = substrate_justifications.into();

		assert_eq!(original_justifications, justifications);
	}

	#[test]
	fn from_substrate_signed_block_to_signed_block_works() {
		let block = "test_string".encode();
		let encoded_justifcation = "test_string".encode();
		let consensus_engine_id: [u8; 4] = [1, 2, 3, 4];
		let justification: Justification = (consensus_engine_id, encoded_justifcation);
		let justifications = Some(Justifications(vec![justification]));

		let signed_block = SignedBlock { block, justifications };

		let substrate_signed_block: sp_runtime::generic::SignedBlock<Vec<u8>> =
			signed_block.clone().into();
		let original_signed_block: SignedBlock<Vec<u8>> = substrate_signed_block.into();

		assert_eq!(original_signed_block, signed_block);
	}
}
