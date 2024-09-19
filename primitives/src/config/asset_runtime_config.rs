// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! Default set of commonly used types by Substrate and Polkadot nodes that use the asset pallet.
//!
//! This file is mostly subxt.
//! https://github.com/paritytech/subxt/blob/ce0a82e3227efb0eae131f025da5f839d9623e15/subxt/src/config/substrate.rs

use crate::{
	config::WithExtrinsicParams, AssetTip, Config, DefaultRuntimeConfig, GenericExtrinsicParams,
};
/// Standard runtime config for Substrate and Polkadot nodes that use the asset pallet.
pub type AssetRuntimeConfig =
	WithExtrinsicParams<DefaultRuntimeConfig, AssetTipExtrinsicParams<DefaultRuntimeConfig>>;

/// A struct representing the signed extra and additional parameters required
/// to construct a transaction and pay in asset fees.
pub type AssetTipExtrinsicParams<T> = GenericExtrinsicParams<T, AssetTip<<T as Config>::Balance>>;
