// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! Polkadot specific configuration
//!
//! This file is mostly subxt.
//! https://github.com/paritytech/subxt/blob/ce0a82e3227efb0eae131f025da5f839d9623e15/subxt/src/config/polkadot.rs

use crate::{
	config::WithExtrinsicParams, Config, GenericExtrinsicParams, PlainTip,
	SubstrateKitchensinkConfig,
};

/// Default set of commonly used types by Polkadot nodes.
pub type PolkadotConfig = WithExtrinsicParams<
	SubstrateKitchensinkConfig,
	PlainTipExtrinsicParams<SubstrateKitchensinkConfig>,
>;

/// A struct representing the signed extra and additional parameters required
/// to construct a transaction and pay in token fees.
pub type PlainTipExtrinsicParams<T> = GenericExtrinsicParams<T, PlainTip<<T as Config>::Balance>>;
