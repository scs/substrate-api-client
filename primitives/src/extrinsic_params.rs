use codec::{Decode, Encode};
use core::{fmt::Debug, marker::PhantomData};
use frame_support::Parameter;
use sp_core::{offchain::storage::InMemOffchainStorage, MaxEncodedLen, H256};
use sp_runtime::{
	generic::Era,
	traits::{
		AtLeast32Bit, CheckEqual, MaybeDisplay, MaybeMallocSizeOf, MaybeSerializeDeserialize,
		Member, SimpleBitOps,
	},
};
use sp_std::prelude::*;

/// Default SignedExtra.
/// Simple generic extra mirroring the SignedExtra currently used in extrinsics.
#[derive(Decode, Encode, Copy, Clone, Eq, PartialEq, Debug)]
pub struct SubstrateDefaultSignedExtra<Tip, Index> {
	pub era: Era,
	#[codec(compact)]
	pub nonce: Index,
	pub tip: Tip,
}

impl<Tip, Index> SubstrateDefaultSignedExtra<Tip, Index> {
	pub fn new(era: Era, nonce: Index, tip: Tip) -> Self {
		Self { era, nonce, tip }
	}
}

/// Default AdditionalSigned fields of the respective SignedExtra fields.
/// The Order is (CheckNonZeroSender, CheckSpecVersion, CheckTxVersion, CheckGenesis, Check::Era, CheckNonce, CheckWeight, transactionPayment::ChargeTransactionPayment).
pub type SubstrateDefaultAdditionalSigned = ((), u32, u32, H256, H256, (), (), ());

/// This trait allows you to configure the "signed extra" and
/// "additional" parameters that are signed and used in transactions.
/// see [`BaseExtrinsicParams`] for an implementation that is compatible with
/// a Polkadot node.
pub trait ExtrinsicParams {
	/// These parameters can be provided to the constructor along with
	/// some default parameters in order to help construct your [`ExtrinsicParams`] object.
	type OtherParams: Default + Clone;

	/// SignedExtra format of the node.
	type SignedExtra: Copy + Encode;

	/// Additional Signed format of the node
	type AdditionalSigned: Encode;

	/// Account index (aka nonce) type. This stores the number of previous transactions
	/// associated with a sender account.
	type Index: Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ Default
		+ MaybeDisplay
		+ AtLeast32Bit
		+ Copy
		+ MaxEncodedLen;

	/// The genesis Hash type. Compatible with substrate.
	type Hash: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ SimpleBitOps
		+ Ord
		+ Default
		+ Copy
		+ CheckEqual
		+ sp_std::hash::Hash
		+ AsRef<[u8]>
		+ AsMut<[u8]>
		+ MaybeMallocSizeOf
		+ MaxEncodedLen;

	/// Construct a new instance of our [`ExtrinsicParams`]
	fn new(
		spec_version: u32,
		transaction_version: u32,
		nonce: Self::Index,
		genesis_hash: Self::Hash,
		other_params: Self::OtherParams,
	) -> Self;

	/// These are the parameters which are sent along with the transaction,
	/// as well as taken into account when signing the transaction.
	fn signed_extra(&self) -> Self::SignedExtra;

	/// These parameters are not sent along with the transaction, but are
	/// taken into account when signing it, meaning the client and node must agree
	/// on their values.
	fn additional_signed(&self) -> Self::AdditionalSigned;
}

/// A struct representing the signed extra and additional parameters required
/// to construct a transaction and pay in asset fees
pub type AssetTipExtrinsicParams = BaseExtrinsicParams<AssetTip, u32, H256>;
/// A builder which leads to [`AssetTipExtrinsicParams`] being constructed.
/// This is what you provide to methods like `sign_and_submit()`.
pub type AssetTipExtrinsicParamsBuilder = BaseExtrinsicParamsBuilder<AssetTip, u32, H256>;

/// A struct representing the signed extra and additional parameters required
/// to construct a transaction and pay in token fees
pub type PlainTipExtrinsicParams = BaseExtrinsicParams<PlainTip, H256>;
/// A builder which leads to [`PlainTipExtrinsicParams`] being constructed.
/// This is what you provide to methods like `sign_and_submit()`.
pub type PlainTipExtrinsicParamsBuilder = BaseExtrinsicParamsBuilder<PlainTip, H256>;

/// An implementation of [`ExtrinsicParams`] that is suitable for constructing
/// extrinsics that can be sent to a node with the same signed extra and additional
/// parameters as a Polkadot/Substrate node.
#[derive(Decode, Encode, Clone, Eq, PartialEq, Debug)]
pub struct BaseExtrinsicParams<Tip, Index, Hash> {
	era: Era,
	nonce: Index,
	tip: Tip,
	spec_version: u32,
	transaction_version: u32,
	genesis_hash: Hash,
	mortality_checkpoint: Hash,
	marker: PhantomData<()>,
}

/// This builder allows you to provide the parameters that can be configured in order to
/// construct a [`BaseExtrinsicParams`] value.
#[derive(Decode, Encode, Copy, Clone, Eq, PartialEq, Debug)]
pub struct BaseExtrinsicParamsBuilder<Tip, Hash> {
	era: Era,
	mortality_checkpoint: Option<Hash>,
	tip: Tip,
}

impl<Tip: Default, Hash> BaseExtrinsicParamsBuilder<Tip, Hash> {
	/// Instantiate the default set of [`BaseExtrinsicParamsBuilder`]
	pub fn new() -> Self {
		Self::default()
	}

	/// Set the [`Era`], which defines how long the transaction will be valid for
	/// (it can be either immortal, or it can be mortal and expire after a certain amount
	/// of time). The second argument is the block hash after which the transaction
	/// becomes valid, and must align with the era phase (see the [`Era::Mortal`] docs
	/// for more detail on that).
	pub fn era(mut self, era: Era, checkpoint: Hash) -> Self {
		self.era = era;
		self.mortality_checkpoint = Some(checkpoint);
		self
	}

	/// Set the tip you'd like to give to the block author
	/// for this transaction.
	pub fn tip(mut self, tip: impl Into<Tip>) -> Self {
		self.tip = tip.into();
		self
	}
}

impl<Tip: Default, Hash> Default for BaseExtrinsicParamsBuilder<Tip, Hash> {
	fn default() -> Self {
		Self { era: Era::Immortal, mortality_checkpoint: None, tip: Tip::default() }
	}
}

impl<Tip, Hash, Index> ExtrinsicParams for BaseExtrinsicParams<Tip, Hash>
where
	u128: From<Tip>,
	Tip: Copy + Default + Encode,
{
	type OtherParams = BaseExtrinsicParamsBuilder<Tip, Hash>;
	type SignedExtra = SubstrateDefaultSignedExtra<Tip, Index>;
	type AdditionalSigned = SubstrateDefaultAdditionalSigned;
	type Hash = Hash;
	type Index = Index;

	fn new(
		spec_version: u32,
		transaction_version: u32,
		nonce: Self::Index,
		genesis_hash: Self::Hash,
		other_params: Self::OtherParams,
	) -> Self {
		BaseExtrinsicParams {
			era: other_params.era,
			tip: other_params.tip,
			spec_version,
			transaction_version,
			genesis_hash,
			mortality_checkpoint: other_params.mortality_checkpoint.unwrap_or(genesis_hash),
			nonce,
			marker: Default::default(),
		}
	}

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
pub struct PlainTip {
	#[codec(compact)]
	tip: u128,
}

impl PlainTip {
	/// Create a new tip of the amount provided.
	pub fn new(amount: u128) -> Self {
		PlainTip { tip: amount }
	}
}

impl From<u128> for PlainTip {
	fn from(n: u128) -> Self {
		PlainTip::new(n)
	}
}

impl From<PlainTip> for u128 {
	fn from(tip: PlainTip) -> Self {
		tip.tip
	}
}

/// A tip payment made in the form of a specific asset.
#[derive(Copy, Clone, Debug, Default, Decode, Encode, Eq, PartialEq)]
pub struct AssetTip {
	#[codec(compact)]
	tip: u128,
	asset: Option<u32>,
}

impl AssetTip {
	/// Create a new tip of the amount provided.
	pub fn new(amount: u128) -> Self {
		AssetTip { tip: amount, asset: None }
	}

	/// Designate the tip as being of a particular asset class.
	/// If this is not set, then the native currency is used.
	pub fn of_asset(mut self, asset: u32) -> Self {
		self.asset = Some(asset);
		self
	}
}

impl From<u128> for AssetTip {
	fn from(n: u128) -> Self {
		AssetTip::new(n)
	}
}

impl From<AssetTip> for u128 {
	fn from(tip: AssetTip) -> Self {
		tip.tip
	}
}
