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

use crate::config::Config;
use codec::{Codec, Decode, Encode};
#[cfg(feature = "disable-metadata-hash-check")]
use primitive_types::H256;
use scale_info::{StaticTypeInfo, TypeInfo};
use sp_runtime::{
	generic::Era,
	impl_tx_ext_default,
	traits::{BlakeTwo256, Dispatchable, Hash, TransactionExtension},
};

/// TxExtension that is compatible with a default Substrate / Polkadot node.
// Unlike the TxExtension on the node side, which seemingly contains a lot more parameters
// see: https://github.com/paritytech/polkadot-sdk/blob/c139739868eddbda495d642219a57602f63c18f5/substrate/bin/node/runtime/src/lib.rs#L2665-L2678
// The TxExtension on the client side mirrors the actual values contained. E.g.
// CheckNonZeroSender does not hold any value inside (see link below)
// https://github.com/paritytech/polkadot-sdk/blob/c139739868eddbda495d642219a57602f63c18f5/substrate/frame/system/src/extensions/check_non_zero_sender.rs#L32
// and is therefore not represented on this side of the TxExtension.
// The Era however is actually defined in the CheckMortality part:
// https://github.com/paritytech/polkadot-sdk/blob/c139739868eddbda495d642219a57602f63c18f5/substrate/frame/system/src/extensions/check_mortality.rs#L39
// and needs to be defined here. Be sure the order matches the one on the node side.
#[derive(Decode, Encode, Copy, Clone, Eq, PartialEq, Debug, TypeInfo)]
pub struct GenericTxExtension<Tip, Index> {
	pub era: Era,
	#[codec(compact)]
	pub nonce: Index,
	pub tip: Tip,
	#[cfg(feature = "disable-metadata-hash-check")]
	pub check_hash: u8,
}

impl<Tip, Index> GenericTxExtension<Tip, Index> {
	pub fn new(era: Era, nonce: Index, tip: Tip) -> Self {
		#[cfg(feature = "disable-metadata-hash-check")]
		{
			Self { era, nonce, tip, check_hash: 0 }
		}
		#[cfg(not(feature = "disable-metadata-hash-check"))]
		{
			Self { era, nonce, tip }
		}
	}
}

impl<Call, Tip, Index> TransactionExtension<Call> for GenericTxExtension<Tip, Index>
where
	Call: Dispatchable,
	GenericTxExtension<Tip, Index>:
		Codec + core::fmt::Debug + Sync + Send + Clone + Eq + PartialEq + StaticTypeInfo,
	Tip: Codec + core::fmt::Debug + Sync + Send + Clone + Eq + PartialEq + StaticTypeInfo,
	Index: Codec + core::fmt::Debug + Sync + Send + Clone + Eq + PartialEq + StaticTypeInfo,
{
	const IDENTIFIER: &'static str = "GenericTxExtension";
	type Implicit = ();
	type Pre = ();
	type Val = ();

	impl_tx_ext_default!(Call; weight validate prepare);
}

/// Default implicit fields of a Polkadot/Substrate node.
/// Order: (CheckNonZeroSender, CheckSpecVersion, CheckTxVersion, CheckGenesis, CheckEra, CheckNonce, CheckWeight, transactionPayment::ChargeTransactionPayment, CheckMetadataHash, WeightReclaim).
// The order and types must match the one defined in the runtime.
// Example: https://github.com/paritytech/polkadot-sdk/blob/c139739868eddbda495d642219a57602f63c18f5/substrate/bin/node/runtime/src/lib.rs#L2665-L2678
// The `Implicit` is the tuple returned from the call TransactionExtension::implicit().
// Each member defined in the `TxExtension` on the node side implements the trait `TransactionExtension`, which
// defines what is returned upon the `implicit` call. The Implicit defined here
// must mirror these return values.
// Example: https://github.com/paritytech/polkadot-sdk/blob/c139739868eddbda495d642219a57602f63c18f5/substrate/frame/system/src/extensions/check_mortality.rs#L62
#[cfg(feature = "disable-metadata-hash-check")]
pub type GenericImplicit<Hash> = ((), u32, u32, Hash, Hash, (), (), (), Option<H256>, ());
#[cfg(not(feature = "disable-metadata-hash-check"))]
pub type GenericImplicit<Hash> = ((), u32, u32, Hash, Hash, (), (), ());

/// This trait allows you to configure the "signed extra" and
/// "additional" parameters that are signed and used in substrate extrinsics.
pub trait ExtrinsicParams<Index, Hash> {
	/// These params represent optional / additional params which are most likely
	/// subject to change. This way, the trait does not need to be adapted if one of
	/// these params is updated.
	type AdditionalParams: Default + Clone;

	/// Extra mirroring the `TxExtension` defined on the node side.
	/// These parameters are sent along with the extrinsic and are taken into account
	/// when signing the extrinsic.
	/// It represents the inner values of the TxExtension, PhantomData is ignored.
	type TxExtension: Copy + Encode;

	/// Implicit format of the node, which is returned upon the call `additional_signed`.
	/// These parameters are not sent along with the extrinsic, but are taken into account
	/// when signing it, meaning the client and node must agree on their values.
	type Implicit: Encode;

	/// Construct a new instance.
	fn new(
		spec_version: u32,
		transaction_version: u32,
		nonce: Index,
		genesis_hash: Hash,
		additional_params: Self::AdditionalParams,
	) -> Self;

	/// Construct the signed extra needed for constructing an extrinsic.
	#[deprecated = "Use transaction_extension instead"]
	fn signed_extra(&self) -> Self::TxExtension {
		self.transaction_extension()
	}

	/// Construct the transaction extension needed for creating an extrinsic.
	fn transaction_extension(&self) -> Self::TxExtension;

	/// Construct any additional data that should be in the signed payload of the extrinsic.
	#[deprecated = "Use implicit instead"]
	fn additional_signed(&self) -> Self::Implicit {
		self.implicit()
	}

	/// Construct any implicit data that should be in the signed payload of the extrinsic.
	fn implicit(&self) -> Self::Implicit;
}

/// An implementation of [`ExtrinsicParams`] that is suitable for constructing
/// extrinsics that can be sent to a node with the same signed extra and additional
/// parameters as a Polkadot/Substrate node.
#[derive(Decode, Encode, Clone, Eq, PartialEq, Debug)]
pub struct GenericExtrinsicParams<T: Config, Tip> {
	era: Era,
	nonce: T::Index,
	tip: Tip,
	spec_version: u32,
	transaction_version: u32,
	genesis_hash: T::Hash,
	mortality_checkpoint: T::Hash,
}

/// Representation of the default Substrate / Polkadot node additional params,
/// needed for constructing an extrinsic with the trait `ExtrinsicParams`.
#[derive(Decode, Encode, Copy, Clone, Eq, PartialEq, Debug)]
pub struct GenericAdditionalParams<Tip, Hash> {
	era: Era,
	mortality_checkpoint: Option<Hash>,
	tip: Tip,
}

impl<Tip: Default, Hash> GenericAdditionalParams<Tip, Hash> {
	/// Instantiate the default set of [`GenericAdditionalParams`]
	pub fn new() -> Self {
		Self::default()
	}

	/// Set the [`Era`], which defines how long the extrinsic will be valid for
	/// (it can be either immortal, or it can be mortal and expire after a certain amount
	/// of time). The second argument is the block hash after which the extrinsic
	/// becomes valid, and must align with the era phase (see the [`Era::Mortal`] docs
	/// for more detail on that).
	pub fn era(mut self, era: Era, checkpoint: Hash) -> Self {
		self.era = era;
		self.mortality_checkpoint = Some(checkpoint);
		self
	}

	/// Set the tip you'd like to give to the block author
	/// for this extrinsic.
	pub fn tip(mut self, tip: impl Into<Tip>) -> Self {
		self.tip = tip.into();
		self
	}
}

impl<Tip: Default, Hash> Default for GenericAdditionalParams<Tip, Hash> {
	fn default() -> Self {
		Self { era: Era::Immortal, mortality_checkpoint: None, tip: Tip::default() }
	}
}

impl<T, Tip> ExtrinsicParams<T::Index, T::Hash> for GenericExtrinsicParams<T, Tip>
where
	T: Config,
	u128: From<Tip>,
	Tip: Copy + Default + Encode,
{
	type AdditionalParams = GenericAdditionalParams<Tip, T::Hash>;
	type TxExtension = GenericTxExtension<Tip, T::Index>;
	type Implicit = GenericImplicit<T::Hash>;

	fn new(
		spec_version: u32,
		transaction_version: u32,
		nonce: T::Index,
		genesis_hash: T::Hash,
		additional_params: Self::AdditionalParams,
	) -> Self {
		GenericExtrinsicParams {
			era: additional_params.era,
			tip: additional_params.tip,
			spec_version,
			transaction_version,
			genesis_hash,
			mortality_checkpoint: additional_params.mortality_checkpoint.unwrap_or(genesis_hash),
			nonce,
		}
	}

	fn transaction_extension(&self) -> Self::TxExtension {
		Self::TxExtension::new(self.era, self.nonce, self.tip)
	}

	fn implicit(&self) -> Self::Implicit {
		#[cfg(feature = "disable-metadata-hash-check")]
		{
			(
				(),
				self.spec_version,
				self.transaction_version,
				self.genesis_hash,
				self.mortality_checkpoint,
				(),
				(),
				(),
				None,
				(),
			)
		}
		#[cfg(not(feature = "disable-metadata-hash-check"))]
		{
			(
				(),
				self.spec_version,
				self.transaction_version,
				self.genesis_hash,
				self.mortality_checkpoint,
				(),
				(),
				(),
			)
		}
	}
}

/// A payload that has been signed for an unchecked extrinsics.
///
/// Note that the payload that we sign to produce unchecked extrinsic signature
/// is going to be different than the `SignaturePayload` - so the thing the extrinsic
/// actually contains.
// https://github.com/paritytech/substrate/blob/1612e39131e3fe57ba4c78447fb1cbf7c4f8830e/primitives/runtime/src/generic/unchecked_extrinsic.rs#L192-L197
#[derive(Decode, Encode, Clone, Eq, PartialEq, Debug)]
pub struct SignedPayload<Call, TransactionExtension, Implicit>(
	(Call, TransactionExtension, Implicit),
);

impl<Call, TransactionExtension, Implicit> SignedPayload<Call, TransactionExtension, Implicit>
where
	Call: Encode,
	TransactionExtension: Encode,
	Implicit: Encode,
{
	/// Create new `SignedPayload` from raw components.
	pub fn from_raw(call: Call, extra: TransactionExtension, additional_signed: Implicit) -> Self {
		Self((call, extra, additional_signed))
	}

	/// Get an encoded version of this payload.
	///
	/// Payloads longer than 256 bytes are going to be `blake2_256`-hashed.
	pub fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.using_encoded(|payload| {
			if payload.len() > 256 {
				f(&BlakeTwo256::hash(payload)[..])
			} else {
				f(payload)
			}
		})
	}
}

/// Default tip payment for a substrate node using the balance pallet.
#[derive(Copy, Clone, Debug, Default, Decode, Encode, Eq, PartialEq)]
pub struct PlainTip<Balance> {
	#[codec(compact)]
	tip: Balance,
}

impl<Balance> PlainTip<Balance> {
	/// Create a new tip of the amount provided.
	pub fn new(amount: Balance) -> Self {
		PlainTip { tip: amount }
	}
}

impl<Balance> From<Balance> for PlainTip<Balance> {
	fn from(n: Balance) -> Self {
		PlainTip::new(n)
	}
}

impl From<PlainTip<u128>> for u128 {
	fn from(tip: PlainTip<u128>) -> Self {
		tip.tip
	}
}

/// Default tip payment for substrate nodes that use the asset payment pallet.
#[derive(Copy, Clone, Debug, Default, Decode, Encode, Eq, PartialEq)]
pub struct AssetTip<Balance> {
	#[codec(compact)]
	tip: Balance,
	asset: Option<u32>,
}

impl<Balance> AssetTip<Balance> {
	/// Create a new tip of the amount provided.
	pub fn new(amount: Balance) -> Self {
		AssetTip { tip: amount, asset: None }
	}

	/// Designate the tip as being of a particular asset class.
	/// If this is not set, then the native currency is used.
	pub fn of_asset(mut self, asset: u32) -> Self {
		self.asset = Some(asset);
		self
	}
}

impl<Balance> From<Balance> for AssetTip<Balance> {
	fn from(n: Balance) -> Self {
		AssetTip::new(n)
	}
}

impl From<AssetTip<u128>> for u128 {
	fn from(tip: AssetTip<u128>) -> Self {
		tip.tip
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_crypto_hashing::blake2_256;

	#[test]
	fn encode_blake2_256_works_as_expected() {
		let bytes = "afaefafe1204udanfai9lfadmlk9aömlsa".as_bytes();
		assert_eq!(&blake2_256(bytes)[..], &BlakeTwo256::hash(bytes)[..]);
	}
}
