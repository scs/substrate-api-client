// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! Substrate specific configuration
//!
//! This file is mostly subxt.
//! https://github.com/paritytech/subxt/blob/ce0a82e3227efb0eae131f025da5f839d9623e15/subxt/src/config/substrate.rs

use crate::{
	config::Config, types::AccountData, AssetTip, BlakeTwo256, ExtrinsicSigner,
	GenericExtrinsicParams, SubstrateBlock, SubstrateHeader, SubstrateOpaqueExtrinsic,
};
use codec::{Decode, Encode};
use core::fmt::Debug;
pub use primitive_types::{H256, U256};
use sp_core::sr25519;
use sp_runtime::{AccountId32, MultiAddress, MultiSignature};

/// Default set of commonly used types by Substrate kitchensink runtime.
#[derive(Decode, Encode, Clone, Eq, PartialEq, Debug)]
pub struct SubstrateKitchensinkConfig {}

impl Config for SubstrateKitchensinkConfig {
	type Index = u32;
	type BlockNumber = u32;
	type Hash = H256;
	type AccountId = AccountId32;
	type Address = MultiAddress<Self::AccountId, u32>;
	type Signature = MultiSignature;
	type Hasher = BlakeTwo256;
	type Header = SubstrateHeader<Self::BlockNumber, BlakeTwo256>;
	type AccountData = AccountData<Self::Balance>;
	type ExtrinsicParams = AssetTipExtrinsicParams<Self>;
	type CryptoKey = sr25519::Pair;
	type ExtrinsicSigner = ExtrinsicSigner<Self>;
	type Block = SubstrateBlock<Self::Header, SubstrateOpaqueExtrinsic>;
	type Balance = u128;
	type ContractCurrency = u128;
	type StakingBalance = u128;
}

/// A struct representing the signed extra and additional parameters required
/// to construct a transaction and pay in asset fees.
pub type AssetTipExtrinsicParams<T> = GenericExtrinsicParams<T, AssetTip<<T as Config>::Balance>>;
