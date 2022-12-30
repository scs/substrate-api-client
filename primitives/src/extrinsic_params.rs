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

use codec::{Decode, Encode};
use core::hash::Hash as HashTrait;
use sp_runtime::{
	generic::Era,
	traits::{BlakeTwo256, Hash},
};

pub type BalanceFor<Runtime> = <Runtime as crate::BalancesConfig>::Balance;
pub type AssetBalanceFor<Runtime> = <Runtime as crate::AssetsConfig>::Balance;
pub type HashFor<Runtime> = <Runtime as crate::FrameSystemConfig>::Hash;
pub type IndexFor<Runtime> = <Runtime as crate::FrameSystemConfig>::Index;

/// A type representing the signed extra and additional parameters required
/// to construct a transaction and pay in asset fees
pub type AssetTipExtrinsicParams<Runtime> =
	DefaultExtrinsicParams<AssetTip<AssetBalanceFor<Runtime>>, IndexFor<Runtime>, HashFor<Runtime>>;

/// A type representing the signed extra and additional parameters required
/// to construct a transaction and pay in token fees
pub type PlainTipExtrinsicParams<Runtime> =
	DefaultExtrinsicParams<PlainTip<BalanceFor<Runtime>>, IndexFor<Runtime>, HashFor<Runtime>>;

/// Default SignedExtra.
/// Simple generic extra mirroring the SignedExtra currently used in extrinsics.
#[derive(Decode, Encode, Copy, Clone, Eq, PartialEq, Debug)]
pub struct DefaultSignedExtra<Tip, Index> {
	pub era: Era,
	#[codec(compact)]
	pub nonce: Index,
	pub tip: Tip,
}

impl<Tip, Index> DefaultSignedExtra<Tip, Index> {
	pub fn new(era: Era, nonce: Index, tip: Tip) -> Self {
		Self { era, nonce, tip }
	}
}

/// Default AdditionalSigned fields of the respective SignedExtra fields.
/// The Order is (CheckNonZeroSender, CheckSpecVersion, CheckTxVersion, CheckGenesis, Check::Era, CheckNonce, CheckWeight, transactionPayment::ChargeTransactionPayment).
pub type DefaultAdditionalSigned<Hash> = ((), u32, u32, Hash, Hash, (), (), ());

/// This trait allows you to configure the "signed extra" and
/// "additional" parameters that are signed and used in transactions.
/// see [`DefaultExtrinsicParams`] for an implementation that is compatible with
/// a Polkadot node.
pub trait ExtrinsicParams<Tip, Index, Hash> {
	/// SignedExtra format of the node.
	type SignedExtra: Copy + Encode;

	/// Additional Signed format of the node
	type AdditionalSigned: Encode;

	/// These are the parameters which are sent along with the transaction,
	/// as well as taken into account when signing the transaction.
	fn signed_extra(&self) -> Self::SignedExtra;

	/// These parameters are not sent along with the transaction, but are
	/// taken into account when signing it, meaning the client and node must agree
	/// on their values.
	fn additional_signed(&self) -> Self::AdditionalSigned;
}

/// An implementation of [`ExtrinsicParams`] that is suitable for constructing
/// extrinsics that can be sent to a node with the same signed extra and additional
/// parameters as a Polkadot/Substrate node.
#[derive(Decode, Encode, Clone, Eq, PartialEq, Debug)]
pub struct DefaultExtrinsicParams<Tip, Index, Hash> {
	era: Era,
	nonce: Index,
	tip: Tip,
	spec_version: u32,
	transaction_version: u32,
	genesis_hash: Hash,
	mortality_checkpoint: Hash,
}

impl<Tip, Index, Hash> DefaultExtrinsicParams<Tip, Index, Hash>
where
	u128: From<Tip>,
	Tip: Copy + Default + Encode,
	Index: Copy + Default + Encode,
	Hash: HashTrait + Encode + Copy,
	DefaultSignedExtra<Tip, Index>: Encode,
{
	pub fn new(
		spec_version: u32,
		transaction_version: u32,
		nonce: Index,
		genesis_hash: Hash,
		era: Era,
		mortality_checkpoint: Option<Hash>,
		tip: Tip,
	) -> Self {
		DefaultExtrinsicParams {
			era,
			tip,
			spec_version,
			transaction_version,
			genesis_hash,
			mortality_checkpoint: mortality_checkpoint.unwrap_or(genesis_hash),
			nonce,
		}
	}
}

impl<Tip, Index, Hash> ExtrinsicParams<Tip, Index, Hash>
	for DefaultExtrinsicParams<Tip, Index, Hash>
where
	u128: From<Tip>,
	Tip: Copy + Default + Encode,
	Index: Copy + Default + Encode,
	Hash: HashTrait + Encode + Copy,
	DefaultSignedExtra<Tip, Index>: Encode,
{
	type SignedExtra = DefaultSignedExtra<Tip, Index>;
	type AdditionalSigned = DefaultAdditionalSigned<Hash>;

	fn signed_extra(&self) -> Self::SignedExtra {
		Self::SignedExtra::new(self.era, self.nonce, self.tip)
	}

	fn additional_signed(&self) -> Self::AdditionalSigned {
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

#[derive(Decode, Encode, Clone, Eq, PartialEq, Debug)]
pub struct SignedPayload<Call, SignedExtra, AdditionalSigned>(
	(Call, SignedExtra, AdditionalSigned),
);

impl<Call, SignedExtra, AdditionalSigned> SignedPayload<Call, SignedExtra, AdditionalSigned>
where
	Call: Encode,
	SignedExtra: Encode,
	AdditionalSigned: Encode,
{
	pub fn from_raw(call: Call, extra: SignedExtra, additional_signed: AdditionalSigned) -> Self {
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

/// A tip payment.
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

/// A tip payment made in the form of a specific asset.
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
