// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! Polkadot specific configuration
//!
//! This file is mostly subxt.
//! https://github.com/paritytech/subxt/blob/ce0a82e3227efb0eae131f025da5f839d9623e15/subxt/src/config/polkadot.rs

use crate::{
	config::WithExtrinsicParams, GenericAdditionalParams, GenericExtrinsicParams, PlainTip,
	SubstrateConfig,
};

/// Default set of commonly used types by Polkadot nodes.
pub type PolkadotConfig =
	WithExtrinsicParams<SubstrateConfig, PolkadotExtrinsicParams<SubstrateConfig>>;

/// A struct representing the signed extra and additional parameters required
/// to construct a transaction for a polkadot node.
pub type PolkadotExtrinsicParams<T> = GenericExtrinsicParams<T, PlainTip<u128>>;

/// A builder which leads to [`PolkadotExtrinsicParams`] being constructed.
/// This is what you provide to methods like `sign_and_submit()`.
pub type PolkadotExtrinsicParamsBuilder<T> = GenericAdditionalParams<T, PlainTip<u128>>;
