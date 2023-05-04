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

use crate::{Block, Hasher, Header};
use alloc::{format, string::String, vec::Vec};
use codec::{Decode, Encode, MaxEncodedLen};
use impl_serde::serialize::{from_hex, FromHexError};
use primitive_types::{H256, U256};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;
use sp_runtime::{traits::Extrinsic, Justification};
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

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct SubstrateOpaqueExtrinsic(Vec<u8>);

impl SubstrateOpaqueExtrinsic {
	/// Convert an encoded extrinsic to an `SubstrateOpaqueExtrinsic`.
	pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, codec::Error> {
		Self::decode(&mut bytes)
	}
}

impl ::serde::Serialize for SubstrateOpaqueExtrinsic {
	fn serialize<S>(&self, seq: S) -> Result<S::Ok, S::Error>
	where
		S: ::serde::Serializer,
	{
		self.using_encoded(|bytes| ::impl_serde::serialize::serialize(bytes, seq))
	}
}

impl<'a> ::serde::Deserialize<'a> for SubstrateOpaqueExtrinsic {
	fn deserialize<D>(de: D) -> Result<Self, D::Error>
	where
		D: ::serde::Deserializer<'a>,
	{
		let r = impl_serde::serialize::deserialize(de)?;
		Decode::decode(&mut &r[..])
			.map_err(|e| ::serde::de::Error::custom(format!("Decode error: {}", e)))
	}
}

impl Extrinsic for SubstrateOpaqueExtrinsic {
	type Call = ();
	type SignaturePayload = ();
}

/// A generic Substrate block type, adapted from `sp_runtime::generic::Block`.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubstrateBlock<H: Header, E: Extrinsic> {
	/// The block header.
	pub header: H,
	/// The accompanying extrinsics.
	pub extrinsics: Vec<E>,
}

impl<H, E> Block for SubstrateBlock<H, E>
where
	H: Header,
	E: Extrinsic + Encode + Serialize,
{
	type Extrinsic = E;
	type Header = H;
	type Hasher = <Self::Header as Header>::Hasher;

	fn header(&self) -> &Self::Header {
		&self.header
	}
	fn extrinsics(&self) -> &[Self::Extrinsic] {
		&self.extrinsics[..]
	}
	fn deconstruct(self) -> (Self::Header, Vec<Self::Extrinsic>) {
		(self.header, self.extrinsics)
	}
	fn new(header: Self::Header, extrinsics: Vec<Self::Extrinsic>) -> Self {
		SubstrateBlock { header, extrinsics }
	}
	fn encode_from(header: &Self::Header, extrinsics: &[Self::Extrinsic]) -> Vec<u8> {
		(header, extrinsics).encode()
	}
}

// Copied from subxt.
// https://github.com/paritytech/subxt/blob/ce0a82e3227efb0eae131f025da5f839d9623e15/subxt/src/config/substrate.rs
/// A type that can hash values using the blaks2_256 algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode)]
pub struct BlakeTwo256;

impl Hasher for BlakeTwo256 {
	type Output = H256;
	fn hash(s: &[u8]) -> Self::Output {
		sp_core_hashing::blake2_256(s).into()
	}
}

/// A generic Substrate header type, adapted from `sp_runtime::generic::Header`.
/// The block number and hasher can be configured to adapt this for other nodes.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubstrateHeader<N: Copy + Into<U256> + TryFrom<U256>, H: Hasher> {
	/// The parent hash.
	pub parent_hash: H::Output,
	/// The block number.
	#[serde(serialize_with = "serialize_number", deserialize_with = "deserialize_number")]
	#[codec(compact)]
	pub number: N,
	/// The state trie merkle root
	pub state_root: H::Output,
	/// The merkle root of the extrinsics.
	pub extrinsics_root: H::Output,
	/// A chain-specific digest of data useful for light clients or referencing auxiliary data.
	pub digest: Digest,
}

impl<N, H> Header for SubstrateHeader<N, H>
where
	N: Copy + Into<u64> + Into<U256> + TryFrom<U256> + Encode,
	H: Hasher + Encode,
	SubstrateHeader<N, H>: Encode,
{
	type Number = N;
	type Hasher = H;
	fn number(&self) -> Self::Number {
		self.number
	}
}

/// Generic header digest. From `sp_runtime::generic::digest`.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
pub struct Digest {
	/// A list of digest items.
	pub logs: Vec<DigestItem>,
}

/// Digest item that is able to encode/decode 'system' digest items and
/// provide opaque access to other items. From `sp_runtime::generic::digest`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DigestItem {
	/// A pre-runtime digest.
	///
	/// These are messages from the consensus engine to the runtime, although
	/// the consensus engine can (and should) read them itself to avoid
	/// code and state duplication. It is erroneous for a runtime to produce
	/// these, but this is not (yet) checked.
	///
	/// NOTE: the runtime is not allowed to panic or fail in an `on_initialize`
	/// call if an expected `PreRuntime` digest is not present. It is the
	/// responsibility of a external block verifier to check this. Runtime API calls
	/// will initialize the block without pre-runtime digests, so initialization
	/// cannot fail when they are missing.
	PreRuntime(ConsensusEngineId, Vec<u8>),

	/// A message from the runtime to the consensus engine. This should *never*
	/// be generated by the native code of any consensus engine, but this is not
	/// checked (yet).
	Consensus(ConsensusEngineId, Vec<u8>),

	/// Put a Seal on it. This is only used by native code, and is never seen
	/// by runtimes.
	Seal(ConsensusEngineId, Vec<u8>),

	/// Some other thing. Unsupported and experimental.
	Other(Vec<u8>),

	/// An indication for the light clients that the runtime execution
	/// environment is updated.
	///
	/// Currently this is triggered when:
	/// 1. Runtime code blob is changed or
	/// 2. `heap_pages` value is changed.
	RuntimeEnvironmentUpdated,
}

// From sp_runtime::generic, DigestItem enum indexes are encoded using this:
#[repr(u32)]
#[derive(Encode, Decode)]
enum DigestItemType {
	Other = 0u32,
	Consensus = 4u32,
	Seal = 5u32,
	PreRuntime = 6u32,
	RuntimeEnvironmentUpdated = 8u32,
}
impl Encode for DigestItem {
	fn encode(&self) -> Vec<u8> {
		let mut v = Vec::new();

		match self {
			Self::Consensus(val, data) => {
				DigestItemType::Consensus.encode_to(&mut v);
				(val, data).encode_to(&mut v);
			},
			Self::Seal(val, sig) => {
				DigestItemType::Seal.encode_to(&mut v);
				(val, sig).encode_to(&mut v);
			},
			Self::PreRuntime(val, data) => {
				DigestItemType::PreRuntime.encode_to(&mut v);
				(val, data).encode_to(&mut v);
			},
			Self::Other(val) => {
				DigestItemType::Other.encode_to(&mut v);
				val.encode_to(&mut v);
			},
			Self::RuntimeEnvironmentUpdated => {
				DigestItemType::RuntimeEnvironmentUpdated.encode_to(&mut v);
			},
		}

		v
	}
}
impl Decode for DigestItem {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let item_type: DigestItemType = Decode::decode(input)?;
		match item_type {
			DigestItemType::PreRuntime => {
				let vals: (ConsensusEngineId, Vec<u8>) = Decode::decode(input)?;
				Ok(Self::PreRuntime(vals.0, vals.1))
			},
			DigestItemType::Consensus => {
				let vals: (ConsensusEngineId, Vec<u8>) = Decode::decode(input)?;
				Ok(Self::Consensus(vals.0, vals.1))
			},
			DigestItemType::Seal => {
				let vals: (ConsensusEngineId, Vec<u8>) = Decode::decode(input)?;
				Ok(Self::Seal(vals.0, vals.1))
			},
			DigestItemType::Other => Ok(Self::Other(Decode::decode(input)?)),
			DigestItemType::RuntimeEnvironmentUpdated => Ok(Self::RuntimeEnvironmentUpdated),
		}
	}
}

/// Consensus engine unique ID. From `sp_runtime::ConsensusEngineId`.
pub type ConsensusEngineId = [u8; 4];

impl serde::Serialize for DigestItem {
	fn serialize<S>(&self, seq: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.using_encoded(|bytes| impl_serde::serialize::serialize(bytes, seq))
	}
}

impl<'a> serde::Deserialize<'a> for DigestItem {
	fn deserialize<D>(de: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let r = impl_serde::serialize::deserialize(de)?;
		Decode::decode(&mut &r[..])
			.map_err(|e| serde::de::Error::custom(format!("Decode error: {e}")))
	}
}

fn serialize_number<S, T: Copy + Into<U256> + TryFrom<U256>>(
	val: &T,
	s: S,
) -> Result<S::Ok, S::Error>
where
	S: serde::Serializer,
{
	let u256: U256 = (*val).into();
	serde::Serialize::serialize(&u256, s)
}

fn deserialize_number<'a, D, T: Copy + Into<U256> + TryFrom<U256>>(d: D) -> Result<T, D::Error>
where
	D: serde::Deserializer<'a>,
{
	// At the time of writing, Smoldot gives back block numbers in numeric rather
	// than hex format. So let's support deserializing from both here:
	use crate::rpc_numbers::NumberOrHex;
	let number_or_hex = NumberOrHex::deserialize(d)?;
	let u256 = number_or_hex.into_u256();
	TryFrom::try_from(u256).map_err(|_| serde::de::Error::custom("Try from failed"))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::UncheckedExtrinsicV4;
	use codec::Encode;
	use core::str::FromStr;
	use node_template_runtime::{BalancesCall, RuntimeCall, SignedExtra};
	use primitive_types::H256;
	use sp_core::crypto::Ss58Codec;
	use sp_runtime::{testing::sr25519, AccountId32, MultiAddress, MultiSignature};

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

	#[test]
	fn deserialize_header_works() {
		let header_json = r#"
            {
                "digest": {
                    "logs": []
                },
                "extrinsicsRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "number": 4,
                "parentHash": "0xcb2690b2c85ceab55be03fc7f7f5f3857e7efeb7a020600ebd4331e10be2f7a5",
                "stateRoot": "0x0000000000000000000000000000000000000000000000000000000000000000"
            }
        "#;

		let header: SubstrateHeader<u32, BlakeTwo256> =
			serde_json::from_str(header_json).expect("valid block header");
		assert_eq!(header.number(), 4);
	}
	#[test]
	fn deserialize_block_works() {
		//header
		let header = SubstrateHeader::<u32, BlakeTwo256> {
			parent_hash: BlakeTwo256::hash(b"1000"),
			number: 2000,
			state_root: BlakeTwo256::hash(b"3000"),
			extrinsics_root: BlakeTwo256::hash(b"4000"),
			digest: Digest { logs: vec![] },
		};

		//extrinsic
		let bob: AccountId32 =
			sr25519::Public::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")
				.unwrap()
				.into();
		let bob = MultiAddress::Id(bob);

		let call1 = RuntimeCall::Balances(BalancesCall::force_transfer {
			source: bob.clone(),
			dest: bob.clone(),
			value: 10,
		});
		let xt1 = UncheckedExtrinsicV4::<
			MultiAddress<AccountId32, u32>,
			RuntimeCall,
			MultiSignature,
			SignedExtra,
		>::new_unsigned(call1.clone());

		//Block
		let extrinsics = vec![xt1];
		let block = SubstrateBlock::new(header.clone(), extrinsics.clone());

		//serialize
		let json = serde_json::to_string(&block).expect("serializing failed");
		let block_des: SubstrateBlock<
			SubstrateHeader<u32, BlakeTwo256>,
			UncheckedExtrinsicV4<
				MultiAddress<AccountId32, u32>,
				RuntimeCall,
				MultiSignature,
				SignedExtra,
			>,
		> = serde_json::from_str(&json).expect("deserializing failed");
		let header_des = block_des.header;
		assert_eq!(header, header_des);
		let extrinsics_des = block_des.extrinsics;
		assert_eq!(extrinsics_des, extrinsics);
	}
}
