// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! Default set of commonly used types by Substrate and Polkadot nodes.
//!
//! This file is mostly subxt.
//! https://github.com/paritytech/subxt/blob/ce0a82e3227efb0eae131f025da5f839d9623e15/subxt/src/config/polkadot.rs

use crate::{
	config::Config, sr25519, types::AccountData, AccountId32, BlakeTwo256, Block, ExtrinsicSigner,
	GenericExtrinsicParams, Header, MultiAddress, MultiSignature, OpaqueExtrinsic, PlainTip, H256,
};
use codec::{Decode, Encode};
use core::fmt::Debug;

/// Standard runtime config for Substrate and Polkadot nodes.
#[derive(Decode, Encode, Clone, Eq, PartialEq, Debug)]
pub struct DefaultRuntimeConfig {}

impl Config for DefaultRuntimeConfig {
	type Index = u32;
	type BlockNumber = u32;
	type Hash = H256;
	type AccountId = AccountId32;
	type Address = MultiAddress<Self::AccountId, u32>;
	type Signature = MultiSignature;
	type Hasher = BlakeTwo256;
	type Header = Header<Self::BlockNumber, BlakeTwo256>;
	type AccountData = AccountData<Self::Balance>;
	type ExtrinsicParams = PlainTipExtrinsicParams<Self>;
	type CryptoKey = sr25519::Pair;
	type ExtrinsicSigner = ExtrinsicSigner<Self>;
	type Block = Block<Self::Header, OpaqueExtrinsic>;
	type Balance = u128;
	type ContractCurrency = u128;
	type StakingBalance = u128;
}

/// A struct representing the signed extra and additional parameters required
/// to construct a transaction and pay in token fees.
pub type PlainTipExtrinsicParams<T> = GenericExtrinsicParams<T, PlainTip<<T as Config>::Balance>>;
