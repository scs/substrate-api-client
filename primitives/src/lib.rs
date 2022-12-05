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

#![cfg_attr(not(feature = "std"), no_std)]

pub use extrinsic_params::*;
pub use extrinsics::*;
#[cfg(not(feature = "std"))]
pub use pallet_traits::*;

#[cfg(feature = "std")]
pub use std_traits::*;
// Configs only need to be reimplemented in no_std mode. No need to do so in std mode.
#[cfg(feature = "std")]
mod std_traits {
	pub use frame_system::Config as FrameSystemConfig;
	pub use pallet_assets::Config as AssetsConfig;
	pub use pallet_balances::Config as BalancesConfig;
}

pub mod extrinsic_params;
pub mod extrinsics;
#[cfg(not(feature = "std"))]
pub mod pallet_traits;

use codec::{Decode, Encode};

/// All balance information for an substrate account.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default)]
pub struct AccountData<Balance> {
	/// Non-reserved part of the balance. There may still be restrictions on this, but it is the
	/// total pool what may in principle be transferred, reserved and used for tipping.
	///
	/// This is the only balance that matters in terms of most operations on tokens. It
	/// alone is used to determine the balance when in the contract execution environment.
	pub free: Balance,
	/// Balance which is reserved and may not be used at all.
	///
	/// This can still get slashed, but gets slashed last of all.
	///
	/// This balance is a 'reserve' balance that other subsystems use in order to set aside tokens
	/// that are still 'owned' by the account holder, but which are suspendable.
	/// This includes named reserve and unnamed reserve.
	pub reserved: Balance,
	/// The amount that `free` may not drop below when withdrawing for *anything except transaction
	/// fee payment*.
	pub misc_frozen: Balance,
	/// The amount that `free` may not drop below when withdrawing specifically for transaction
	/// fee payment.
	pub fee_frozen: Balance,
}
