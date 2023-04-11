/*
   Copyright 2019 Supercomputing Systems AG

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

	   http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.

*/

//! Re-defintion of substrate primitives.
//! Needed because substrate pallets compile to wasm in no_std.

use alloc::string::String;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;
use sp_runtime::traits::{AtLeast32BitUnsigned, Zero};

/// Type used to encode the number of references an account has.
pub type RefCount = u32;
/// Information of an account.
// https://github.com/paritytech/substrate/blob/416a331399452521f3e9cf7e1394d020373a95c5/frame/system/src/lib.rs#L735-L753
#[derive(Clone, Eq, PartialEq, Default, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct AccountInfo<Index, AccountData> {
	/// The number of transactions this account has sent.
	pub nonce: Index,
	/// The number of other modules that currently depend on this account's existence. The account
	/// cannot be reaped until this is zero.
	pub consumers: RefCount,
	/// The number of other modules that allow this account to exist. The account may not be reaped
	/// until this and `sufficients` are both zero.
	pub providers: RefCount,
	/// The number of modules that allow this account to exist for their own purposes only. The
	/// account may not be reaped until this and `providers` are both zero.
	pub sufficients: RefCount,
	/// The additional data that belongs to this account. Used to store the balance(s) in a lot of
	/// chains.
	pub data: AccountData,
}

/// The base fee and adjusted weight and length fees constitute the _inclusion fee_.
// https://github.com/paritytech/substrate/blob/a1c1286d2ca6360a16d772cc8bea2190f77f4d8f/frame/transaction-payment/src/types.rs#L29-L60
#[derive(Encode, Decode, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InclusionFee<Balance> {
	/// This is the minimum amount a user pays for a transaction. It is declared
	/// as a base _weight_ in the runtime and converted to a fee using `WeightToFee`.
	pub base_fee: Balance,
	/// The length fee, the amount paid for the encoded length (in bytes) of the transaction.
	pub len_fee: Balance,
	///
	/// - `targeted_fee_adjustment`: This is a multiplier that can tune the final fee based on the
	///   congestion of the network.
	/// - `weight_fee`: This amount is computed based on the weight of the transaction. Weight
	/// accounts for the execution time of a transaction.
	///
	/// adjusted_weight_fee = targeted_fee_adjustment * weight_fee
	pub adjusted_weight_fee: Balance,
}

impl<Balance: AtLeast32BitUnsigned + Copy> InclusionFee<Balance> {
	/// Returns the total of inclusion fee.
	///
	/// ```ignore
	/// inclusion_fee = base_fee + len_fee + adjusted_weight_fee
	/// ```
	pub fn inclusion_fee(&self) -> Balance {
		self.base_fee
			.saturating_add(self.len_fee)
			.saturating_add(self.adjusted_weight_fee)
	}
}

/// The `FeeDetails` is composed of:
///   - (Optional) `inclusion_fee`: Only the `Pays::Yes` transaction can have the inclusion fee.
///   - `tip`: If included in the transaction, the tip will be added on top. Only signed
///     transactions can have a tip.
// https://github.com/paritytech/substrate/blob/a1c1286d2ca6360a16d772cc8bea2190f77f4d8f/frame/transaction-payment/src/types.rs#L62-L90
#[derive(Encode, Decode, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeeDetails<Balance> {
	/// The minimum fee for a transaction to be included in a block.
	pub inclusion_fee: Option<InclusionFee<Balance>>,
	#[serde(skip)]
	pub tip: Balance,
}

impl<Balance: AtLeast32BitUnsigned + Copy> FeeDetails<Balance> {
	/// Returns the final fee.
	///
	/// ```ignore
	/// final_fee = inclusion_fee + tip;
	/// ```
	pub fn final_fee(&self) -> Balance {
		self.inclusion_fee
			.as_ref()
			.map(|i| i.inclusion_fee())
			.unwrap_or_else(|| Zero::zero())
			.saturating_add(self.tip)
	}
}

/// Information related to a dispatchable's class, weight, and fee that can be queried from the
/// runtime.
// https://github.com/paritytech/substrate/blob/a1c1286d2ca6360a16d772cc8bea2190f77f4d8f/frame/transaction-payment/src/types.rs#L92-L116
#[derive(Eq, PartialEq, Encode, Decode, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(serialize = "Balance: core::fmt::Display, Weight: Serialize"))]
#[serde(bound(deserialize = "Balance: core::str::FromStr, Weight: Deserialize<'de>"))]
pub struct RuntimeDispatchInfo<Balance, Weight = crate::serde_impls::Weight> {
	/// Weight of this dispatch.
	pub weight: Weight,
	/// Class of this dispatch.
	pub class: DispatchClass,
	/// The inclusion fee of this dispatch.
	///
	/// This does not include a tip or anything else that
	/// depends on the signature (i.e. depends on a `SignedExtension`).
	#[serde(with = "serde_balance")]
	pub partial_fee: Balance,
}

mod serde_balance {
	use alloc::string::ToString;
	use serde::{Deserialize, Deserializer, Serializer};

	pub fn serialize<S: Serializer, T: core::fmt::Display>(
		t: &T,
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&t.to_string())
	}

	pub fn deserialize<'de, D: Deserializer<'de>, T: core::str::FromStr>(
		deserializer: D,
	) -> Result<T, D::Error> {
		let s = alloc::string::String::deserialize(deserializer)?;
		s.parse::<T>().map_err(|_| serde::de::Error::custom("Parse from string failed"))
	}
}

/// A generalized group of dispatch types.
///
/// NOTE whenever upgrading the enum make sure to also update
/// [DispatchClass::all] and [DispatchClass::non_mandatory] helper functions.
// https://github.com/paritytech/substrate/blob/a1c1286d2ca6360a16d772cc8bea2190f77f4d8f/frame/support/src/dispatch.rs#L133-L177
#[derive(
	PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum DispatchClass {
	/// A normal dispatch.
	Normal,
	/// An operational dispatch.
	Operational,
	/// A mandatory dispatch. These kinds of dispatch are always included regardless of their
	/// weight, therefore it is critical that they are separately validated to ensure that a
	/// malicious validator cannot craft a valid but impossibly heavy block. Usually this just
	/// means ensuring that the extrinsic can only be included once and that it is always very
	/// light.
	///
	/// Do *NOT* use it for extrinsics that can be heavy.
	///
	/// The only real use case for this is inherent extrinsics that are required to execute in a
	/// block for the block to be valid, and it solves the issue in the case that the block
	/// initialization is sufficiently heavy to mean that those inherents do not fit into the
	/// block. Essentially, we assume that in these exceptional circumstances, it is better to
	/// allow an overweight block to be created than to not allow any block at all to be created.
	Mandatory,
}

impl Default for DispatchClass {
	fn default() -> Self {
		Self::Normal
	}
}

impl DispatchClass {
	/// Returns an array containing all dispatch classes.
	pub fn all() -> &'static [DispatchClass] {
		&[DispatchClass::Normal, DispatchClass::Operational, DispatchClass::Mandatory]
	}

	/// Returns an array of all dispatch classes except `Mandatory`.
	pub fn non_mandatory() -> &'static [DispatchClass] {
		&[DispatchClass::Normal, DispatchClass::Operational]
	}
}

/// A destination account for payment.
#[derive(
	PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, Default,
)]
pub enum RewardDestination<AccountId> {
	/// Pay into the stash account, increasing the amount at stake accordingly.
	#[default]
	Staked,
	/// Pay into the stash account, not increasing the amount at stake.
	Stash,
	/// Pay into the controller account.
	Controller,
	/// Pay into a specified account.
	Account(AccountId),
	/// Receive no reward.
	None,
}

/// Health struct returned by the RPC
// https://github.com/paritytech/substrate/blob/c172d0f683fab3792b90d876fd6ca27056af9fe9/client/rpc-api/src/system/helpers.rs#L40-L58
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Health {
	/// Number of connected peers
	pub peers: usize,
	/// Is the node syncing
	pub is_syncing: bool,
	/// Should this node have any peers
	///
	/// Might be false for local chains or when running without discovery.
	pub should_have_peers: bool,
}

impl core::fmt::Display for Health {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(fmt, "{} peers ({})", self.peers, if self.is_syncing { "syncing" } else { "idle" })
	}
}

/// The type of a chain.
///
/// This can be used by tools to determine the type of a chain for displaying
/// additional information or enabling additional features.
// https://github.com/paritytech/substrate/blob/c172d0f683fab3792b90d876fd6ca27056af9fe9/client/chain-spec/src/lib.rs#L193-L207
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ChainType {
	/// A development chain that runs mainly on one node.
	Development,
	/// A local chain that runs locally on multiple nodes for testing purposes.
	Local,
	/// A live chain.
	Live,
	/// Some custom chain type.
	Custom(String),
}

/// Arbitrary properties defined in chain spec as a JSON object
// https://github.com/paritytech/substrate/blob/c172d0f683fab3792b90d876fd6ca27056af9fe9/client/chain-spec/src/lib.rs#L215-L216
pub type Properties = serde_json::map::Map<String, serde_json::Value>;
