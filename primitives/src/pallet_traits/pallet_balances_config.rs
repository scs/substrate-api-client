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
use codec::{Codec, EncodeLike, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Get, MaybeSerializeDeserialize, Member},
	FixedPointOperand,
};

/// Simplifed pallet balances Config trait. Needed because substrate pallets compile to wasm
/// in no_std mode.
pub trait BalancesConfig: crate::FrameSystemConfig {
	type Balance: Codec
		+ EncodeLike
		+ Member
		+ AtLeast32BitUnsigned
		+ Default
		+ Copy
		+ MaybeSerializeDeserialize
		+ MaxEncodedLen
		+ TypeInfo
		+ FixedPointOperand;
	type DustRemoval;
	type RuntimeEvent;
	type ExistentialDeposit: Get<Self::Balance>;
	type AccountStore;
	type WeightInfo;
	type MaxLocks: Get<u32>;
	type MaxReserves: Get<u32>;
	type ReserveIdentifier: Codec + EncodeLike + TypeInfo + Member + MaxEncodedLen + Ord + Copy;
}

#[cfg(feature = "std")]
impl<T> BalancesConfig for T
where
	T: pallet_balances::Config,
{
	type Balance = T::Balance;
	type DustRemoval = T::DustRemoval;
	type RuntimeEvent = <T as pallet_balances::Config>::RuntimeEvent;
	type ExistentialDeposit = T::ExistentialDeposit;
	type AccountStore = T::AccountStore;
	type WeightInfo = T::WeightInfo;
	type MaxLocks = T::MaxLocks;
	type MaxReserves = T::MaxReserves;
	type ReserveIdentifier = T::ReserveIdentifier;
}
