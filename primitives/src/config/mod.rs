// This file was taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! The config used by the API.
//!
//! This file is mostly subxt:
//! https://github.com/paritytech/subxt/blob/ce0a82e3227efb0eae131f025da5f839d9623e15/subxt/src/config/mod.rs

use codec::{Decode, Encode, FullCodec};
use core::{fmt::Debug, marker::PhantomData};
use serde::{de::DeserializeOwned, Serialize};
use sp_core::Pair;
use sp_runtime::traits::{
	AtLeast32Bit, AtLeast32BitUnsigned, Block, Hash as HashTrait, Header as HeaderTrait,
	MaybeSerializeDeserialize,
};

use crate::{extrinsic_params, ExtrinsicSigner, SignExtrinsic};

pub use asset_runtime_config::*;
pub use default_runtime_config::*;

pub mod asset_runtime_config;
pub mod default_runtime_config;

/// Runtime types.
pub trait Config {
	/// Account index (aka nonce) type. This stores the number of previous
	/// transactions associated with a sender account.
	/// This type enforces the (de)serialization implementation
	/// also in no-std mode (unlike substrates MaybeSerializeDeserialize).
	type Index: Default + Debug + Copy + DeserializeOwned + AtLeast32Bit + Decode;

	/// The block number type used by the runtime.
	type BlockNumber: Debug
		+ Copy
		+ Encode
		+ Default
		+ Serialize
		+ DeserializeOwned
		+ core::hash::Hash
		+ core::str::FromStr
		+ Into<u64>
		+ AtLeast32BitUnsigned;

	/// The output of the `Hashing` function.
	type Hash: Debug
		+ Copy
		+ Send
		+ Sync
		+ Decode
		+ Default
		+ AsRef<[u8]>
		+ Serialize
		+ DeserializeOwned
		+ Encode
		+ PartialEq;

	/// The account ID type.
	type AccountId: Debug
		+ Clone
		+ Encode
		+ MaybeSerializeDeserialize
		+ From<<Self::CryptoKey as Pair>::Public>;

	/// The address type.
	type Address: Debug + Clone + Encode + From<Self::AccountId>; //type Lookup: StaticLookup<Target = Self::AccountId>;

	/// The signature type.
	type Signature: Debug + Encode + From<<Self::CryptoKey as Pair>::Signature>;

	/// The hashing system (algorithm) being used in the runtime (e.g. Blake2).
	type Hasher: Debug + HashTrait<Output = Self::Hash>;

	/// The block header.
	type Header: Debug
		+ HeaderTrait<Number = Self::BlockNumber, Hashing = Self::Hasher>
		+ Send
		+ DeserializeOwned;

	/// The account data.
	type AccountData: Debug + Clone + FullCodec;

	/// This type defines the extrinsic extra and additional parameters.
	type ExtrinsicParams: extrinsic_params::ExtrinsicParams<Self::Index, Self::Hash>;

	/// The cryptographic PKI key pair type used to sign the extrinsic
	type CryptoKey: Pair;

	/// This extrinsic signer.
	type ExtrinsicSigner: SignExtrinsic<Self::AccountId>;

	/// The block type.
	type Block: Block + DeserializeOwned;

	/// The balance type.
	type Balance: Debug
		+ Decode
		+ Encode
		+ AtLeast32BitUnsigned
		+ Default
		+ Copy
		+ Serialize
		+ DeserializeOwned;

	/// The currency type of the contract pallet.
	type ContractCurrency: Debug
		+ Decode
		+ Encode
		+ AtLeast32BitUnsigned
		+ Default
		+ Copy
		+ Serialize
		+ DeserializeOwned;

	/// The balance type of the staking pallet.
	type StakingBalance: Debug
		+ Decode
		+ Encode
		+ AtLeast32BitUnsigned
		+ Default
		+ Copy
		+ Serialize
		+ DeserializeOwned;
}

/// Take a type implementing [`Config`] (eg [`AssetRuntimeConfig`]), and some type which describes the
/// additional and extra parameters to pass to an extrinsic (see [`ExtrinsicParams`]),
/// and returns a type implementing [`Config`] with those new [`ExtrinsicParams`].
///
/// # Example
///
/// ```
/// use ac_primitives::{ AssetRuntimeConfig, WithExtrinsicParams, PlainTipExtrinsicParams };
///
/// // This is how DefaultRuntimeConfig is implemented:
/// type DefaultRuntimeConfig = WithExtrinsicParams<AssetRuntimeConfig, PlainTipExtrinsicParams<AssetRuntimeConfig>>;
/// ```
#[derive(Decode, Encode, Clone, Eq, PartialEq, Debug)]
pub struct WithExtrinsicParams<T: Config, E: extrinsic_params::ExtrinsicParams<T::Index, T::Hash>> {
	_marker: PhantomData<(T, E)>,
}

impl<T: Config, E: extrinsic_params::ExtrinsicParams<T::Index, T::Hash>> Config
	for WithExtrinsicParams<T, E>
{
	type Index = T::Index;
	type BlockNumber = T::BlockNumber;
	type Hash = T::Hash;
	type AccountId = T::AccountId;
	type Address = T::Address;
	type Signature = T::Signature;
	type Hasher = T::Hasher;
	type Header = T::Header;
	type AccountData = T::AccountData;
	type ExtrinsicParams = E;
	type CryptoKey = T::CryptoKey;
	type ExtrinsicSigner = ExtrinsicSigner<Self>;
	type Block = T::Block;
	type Balance = T::Balance;
	type ContractCurrency = T::ContractCurrency;
	type StakingBalance = T::StakingBalance;
}
