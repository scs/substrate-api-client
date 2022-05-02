use codec::{Compact, Decode, Encode};
use sp_core::{blake2_256, H256};
use sp_runtime::generic::Era;
use sp_std::prelude::*;

/// Simple generic extra mirroring the SignedExtra currently used in extrinsics. Does not implement
/// the SignedExtension trait. It simply encodes to the same bytes as the real SignedExtra. The
/// Order is (CheckVersion, CheckGenesis, Check::Era, CheckNonce, CheckWeight, transactionPayment::ChargeTransactionPayment).
/// This can be locked up in the System module. Fields that are merely PhantomData are not encoded and are
/// therefore omitted here.
#[derive(Decode, Encode, Clone, Eq, PartialEq, Debug)]
pub struct GenericExtra(pub Era, pub Compact<u32>, pub Compact<u128>);

/// This trait allows you to configure the "signed extra" and
/// "additional" parameters that are signed and used in transactions.
/// see [`BaseExtrinsicParams`] for an implementation that is compatible with
/// a Polkadot node.
pub trait ExtrinsicParams {
    /// These parameters can be provided to the constructor along with
    /// some default parameters that `subxt` understands, in order to
    /// help construct your [`ExtrinsicParams`] object.
    type OtherParams;

    /// Construct a new instance of our [`ExtrinsicParams`]
    fn new(nonce: u32, other_params: Self::OtherParams) -> Self;

    /// This is expected to SCALE encode the "signed extra" parameters
    /// to some buffer that has been provided. These are the parameters
    /// which are sent along with the transaction, as well as taken into
    /// account when signing the transaction.
    fn encode_extra_to(&self, v: &mut Vec<u8>);
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

#[derive(Decode, Encode, Clone, Eq, PartialEq, Debug)]
pub struct BaseExtrinsicParams<Tip> {
    era: Era,
    nonce: u32,
    tip: Tip,
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

/// Get the generic extra from the BaseExtrinsicParams.
impl<Tip> From<BaseExtrinsicParams<Tip>> for GenericExtra
where
    u128: From<Tip>,
{
    fn from(p: BaseExtrinsicParams<Tip>) -> GenericExtra {
        let BaseExtrinsicParams { era, nonce, tip } = p;
        GenericExtra(era, Compact(nonce), Compact(tip.into()))
    }
}

impl<Tip: Encode> ExtrinsicParams for BaseExtrinsicParams<Tip> {
    type OtherParams = BaseExtrinsicParamsBuilder<Tip>;

    fn new(nonce: u32, other_params: Self::OtherParams) -> Self {
        BaseExtrinsicParams {
            era: other_params.era,
            tip: other_params.tip,
            nonce,
        }
    }

    fn encode_extra_to(&self, v: &mut Vec<u8>) {
        let nonce: u64 = self.nonce.into();
        let tip = self.tip.encode(); //?
        (self.era, Compact(nonce), tip).encode_to(v);
    }
}

/// additionalSigned fields of the respective SignedExtra fields.
/// Order is the same as declared in the extra.
pub type AdditionalSigned = (u32, u32, H256, H256, (), (), ());

#[derive(Decode, Encode, Clone, Eq, PartialEq, Debug)]
pub struct SignedPayload<Call>((Call, GenericExtra, AdditionalSigned));

impl<Call> SignedPayload<Call>
where
    Call: Encode,
{
    pub fn from_raw(call: Call, extra: GenericExtra, additional_signed: AdditionalSigned) -> Self {
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
