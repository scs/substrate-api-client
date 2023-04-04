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

use crate::extrinsic_params;
use codec::{Decode, Encode, FullCodec};
use core::{fmt::Debug, marker::PhantomData};
use serde::{de::DeserializeOwned, Serialize};
use sp_runtime::traits::{AtLeast32Bit, AtLeast32BitUnsigned, MaybeSerializeDeserialize};

/// Runtime types.
pub trait Config {
	/// Account index (aka nonce) type. This stores the number of previous
	/// transactions associated with a sender account.
	/// This type enforces the (de)serialization implementation
	/// also in no-std mode (unlike substrates MaybeSerializeDeserialize).
	type Index: Debug + Copy + DeserializeOwned + AtLeast32Bit + Decode;

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
	type AccountId: Debug + Clone + Encode + MaybeSerializeDeserialize;

	/// The address type.
	type Address: Debug + Clone + Encode + From<Self::AccountId>; //type Lookup: StaticLookup<Target = Self::AccountId>;

	/// The signature type.
	type Signature: Debug + Encode;

	/// The hashing system (algorithm) being used in the runtime (e.g. Blake2).
	type Hasher: Debug + Hasher<Output = Self::Hash>;

	/// The block header.
	type Header: Debug + Header<Hasher = Self::Hasher> + Send + DeserializeOwned;

	/// The account data.
	type AccountData: Debug + Clone + FullCodec;

	/// This type defines the extrinsic extra and additional parameters.
	type ExtrinsicParams: extrinsic_params::ExtrinsicParams<Self::Index, Self::Hash>;

	/// The balance type.
	type Balance: Debug
		+ Decode
		+ Encode
		+ AtLeast32BitUnsigned
		+ Default
		+ Copy
		+ Serialize
		+ DeserializeOwned;

	/// The balance type of the contract pallet.
	type ContractBalance: Debug
		+ Decode
		+ Encode
		+ AtLeast32BitUnsigned
		+ Default
		+ Copy
		+ Serialize
		+ DeserializeOwned;
}

/// This represents the hasher used by a node to hash things like block headers
/// and extrinsics.
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
pub trait Header: Sized + Encode {
	/// The block number type for this header.
	type Number: Into<u64>;
	/// The hasher used to hash this header.
	type Hasher: Hasher;

	/// Return the block number of this header.
	fn number(&self) -> Self::Number;

	/// Hash this header.
	fn hash(&self) -> <Self::Hasher as Hasher>::Output {
		Self::Hasher::hash_of(self)
	}
}

/// Take a type implementing [`Config`] (eg [`SubstrateConfig`]), and some type which describes the
/// additional and extra parameters to pass to an extrinsic (see [`ExtrinsicParams`]),
/// and returns a type implementing [`Config`] with those new [`ExtrinsicParams`].
///
/// # Example
///
/// ```
/// use subxt::config::{ SubstrateConfig, WithExtrinsicParams, polkadot::PolkadotExtrinsicParams };
///
/// // This is how PolkadotConfig is implemented:
/// type PolkadotConfig = WithExtrinsicParams<SubstrateConfig, PolkadotExtrinsicParams<SubstrateConfig>>;
/// ```
pub struct WithExtrinsicParams<T: Config, E: extrinsic_params::ExtrinsicParams<T::Index, T::Hash>> {
	_marker: PhantomData<(T, E)>,
}

impl<T: Config, E: extrinsic_params::ExtrinsicParams<T::Index, T::Hash>> Config
	for WithExtrinsicParams<T, E>
{
	type Index = T::Index;
	type Hash = T::Hash;
	type AccountId = T::AccountId;
	type Address = T::Address;
	type Signature = T::Signature;
	type Hasher = T::Hasher;
	type Header = T::Header;
	type AccountData = T::AccountData;
	type ExtrinsicParams = E;
	type Balance = T::Balance;
	type ContractBalance = T::ContractBalance;
}
