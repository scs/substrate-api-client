use codec::{Decode, Encode};
use core::marker::PhantomData;
use sp_core::{blake2_256, H256};
use sp_runtime::generic::Era;
use sp_std::prelude::*;

/// Default SignedExtra.
/// Simple generic extra mirroring the SignedExtra currently used in extrinsics.
#[derive(Decode, Encode, Copy, Clone, Eq, PartialEq, Debug)]
pub struct SubstrateDefaultSignedExtra<Tip> {
    pub era: Era,
    #[codec(compact)]
    pub nonce: u32,
    pub tip: Tip,
}

impl<Tip> SubstrateDefaultSignedExtra<Tip> {
    pub fn new(era: Era, nonce: u32, tip: Tip) -> Self {
        Self { era, nonce, tip }
    }
}

/// Default AdditionalSigned fields of the respective SignedExtra fields.
/// The Order is (CheckSpecVersion, CheckTxVersion, CheckGenesis, Check::Era, CheckNonce, CheckWeight, transactionPayment::ChargeTransactionPayment).
pub type SubstrateDefaultAdditionalSigned = (u32, u32, H256, H256, (), (), ());

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

    /// Construct a new instance of our [`ExtrinsicParams`]
    fn new(
        spec_version: u32,
        transaction_version: u32,
        nonce: u32,
        genesis_hash: H256,
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
pub type AssetTipExtrinsicParams = BaseExtrinsicParams<AssetTip>;
/// A builder which leads to [`AssetTipExtrinsicParams`] being constructed.
/// This is what you provide to methods like `sign_and_submit()`.
pub type AssetTipExtrinsicParamsBuilder = BaseExtrinsicParamsBuilder<AssetTip>;

/// A struct representing the signed extra and additional parameters required
/// to construct a transaction and pay in token fees
pub type PlainTipExtrinsicParams = BaseExtrinsicParams<PlainTip>;
/// A builder which leads to [`PlainTipExtrinsicParams`] being constructed.
/// This is what you provide to methods like `sign_and_submit()`.
pub type PlainTipExtrinsicParamsBuilder = BaseExtrinsicParamsBuilder<PlainTip>;

/// An implementation of [`ExtrinsicParams`] that is suitable for constructing
/// extrinsics that can be sent to a node with the same signed extra and additional
/// parameters as a Polkadot/Substrate node.
#[derive(Decode, Encode, Clone, Eq, PartialEq, Debug)]
pub struct BaseExtrinsicParams<Tip> {
    era: Era,
    nonce: u32,
    tip: Tip,
    spec_version: u32,
    transaction_version: u32,
    genesis_hash: H256,
    mortality_checkpoint: H256,
    marker: PhantomData<()>,
}

/// This builder allows you to provide the parameters that can be configured in order to
/// construct a [`BaseExtrinsicParams`] value.
#[derive(Decode, Encode, Copy, Clone, Eq, PartialEq, Debug)]
pub struct BaseExtrinsicParamsBuilder<Tip> {
    era: Era,
    mortality_checkpoint: Option<H256>,
    tip: Tip,
}

impl<Tip: Default> BaseExtrinsicParamsBuilder<Tip> {
    /// Instantiate the default set of [`BaseExtrinsicParamsBuilder`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the [`Era`], which defines how long the transaction will be valid for
    /// (it can be either immortal, or it can be mortal and expire after a certain amount
    /// of time). The second argument is the block hash after which the transaction
    /// becomes valid, and must align with the era phase (see the [`Era::Mortal`] docs
    /// for more detail on that).
    pub fn era(mut self, era: Era, checkpoint: H256) -> Self {
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

impl<Tip: Default> Default for BaseExtrinsicParamsBuilder<Tip> {
    fn default() -> Self {
        Self {
            era: Era::Immortal,
            mortality_checkpoint: None,
            tip: Tip::default(),
        }
    }
}

impl<Tip: Encode> ExtrinsicParams for BaseExtrinsicParams<Tip>
where
    u128: From<Tip>,
    Tip: Copy + Default,
{
    type OtherParams = BaseExtrinsicParamsBuilder<Tip>;
    type SignedExtra = SubstrateDefaultSignedExtra<Tip>;
    type AdditionalSigned = SubstrateDefaultAdditionalSigned;

    fn new(
        spec_version: u32,
        transaction_version: u32,
        nonce: u32,
        genesis_hash: H256,
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
                f(&blake2_256(payload)[..])
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
        AssetTip {
            tip: amount,
            asset: None,
        }
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
