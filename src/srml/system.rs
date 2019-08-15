//! Implements support for the srml_system module.
use codec::Codec;
use runtime_primitives::traits::{
    Bounded,
    CheckEqual,
    Hash,
    Header,
    MaybeDisplay,
    MaybeSerializeDebug,
    MaybeSerializeDebugButNotDeserialize,
    Member,
    SignedExtension,
    SimpleArithmetic,
    SimpleBitOps,
    StaticLookup,
};
use runtime_support::Parameter;
use serde::de::DeserializeOwned;
use system::Event;
use primitives::Pair;

/// The subset of the `srml_system::Trait` that a client must implement.
pub trait System {
    /// Account index (aka nonce) type. This stores the number of previous
    /// transactions associated with a sender account.
    type Index: Parameter
    + Member
    + MaybeSerializeDebugButNotDeserialize
    + Default
    + MaybeDisplay
    + SimpleArithmetic
    + Copy;

    /// The block number type used by the runtime.
    type BlockNumber: Parameter
    + Member
    + MaybeSerializeDebug
    + MaybeDisplay
    + SimpleArithmetic
    + Default
    + Bounded
    + Copy
    + std::hash::Hash;

    /// The output of the `Hashing` function.
    type Hash: Parameter
    + Member
    + MaybeSerializeDebug
    + MaybeDisplay
    + SimpleBitOps
    + Default
    + Copy
    + CheckEqual
    + std::hash::Hash
    + AsRef<[u8]>
    + AsMut<[u8]>;

    /// The hashing system (algorithm) being used in the runtime (e.g. Blake2).
    type Hashing: Hash<Output = Self::Hash>;

    /// The user account identifier type for the runtime.
    type AccountId: Parameter
    + Member
    + MaybeSerializeDebug
    + MaybeDisplay
    + Ord
    + Default;

    /// Converting trait to take a source type and convert to `AccountId`.
    ///
    /// Used to define the type and conversion mechanism for referencing
    /// accounts in transactions. It's perfectly reasonable for this to be an
    /// identity conversion (with the source type being `AccountId`), but other
    /// modules (e.g. Indices module) may provide more functional/efficient
    /// alternatives.
    type Lookup: StaticLookup<Target = Self::AccountId>;

    /// The block header.
    type Header: Parameter
    + Header<Number = Self::BlockNumber, Hash = Self::Hash>
    + DeserializeOwned;

    /// The aggregated event type of the runtime.
    type Event: Parameter + Member + From<Event>;

    /// The `SignedExtension` to the basic transaction logic.
    type SignedExtra: SignedExtension;

    /// Creates the `SignedExtra` from the account nonce.
    fn extra(nonce: Self::Index) -> Self::SignedExtra;
}