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
use codec::{Codec, Decode, Encode, EncodeLike, FullCodec, MaxEncodedLen};
use core::fmt::Debug;
use frame_support::{
	traits::{
		ConstU32, Contains, EnsureOrigin, Get, HandleLifetime, OnKilledAccount, OnNewAccount,
		OriginTrait, PalletInfo, SortedMembers, StoredMap, TypedGet,
	},
	Parameter,
};
use scale_info::TypeInfo;
use sp_core::storage::well_known_keys;
use sp_runtime::{
	traits::{
		self, AtLeast32Bit, AtLeast32BitUnsigned, BadOrigin, BlockNumberProvider, Bounded,
		CheckEqual, Dispatchable, Hash, Lookup, LookupError, MaybeDisplay, MaybeMallocSizeOf,
		MaybeSerializeDeserialize, Member, One, Saturating, SimpleBitOps, StaticLookup, Zero,
	},
	FixedPointOperand,
};

/// Simplifed Frame system Config trait. Needed because substrate pallets compile to wasm
/// in no_std mode.
pub trait FrameSystemConfig {
	/// The basic call filter to use in Origin. All origins are built with this filter as base,
	/// except Root.
	type BaseCallFilter;

	/// Block & extrinsics weights: base values and limits.
	type BlockWeights;

	/// The maximum length of a block (in bytes).
	type BlockLength;

	/// The `RuntimeOrigin` type used by dispatchable calls.
	type RuntimeOrigin: Clone;

	/// The aggregated `RuntimeCall` type.
	type RuntimeCall: Parameter + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin> + Debug;

	type Index: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ Default
		+ MaybeDisplay
		+ AtLeast32Bit
		+ Copy
		+ MaxEncodedLen;

	type BlockNumber: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ AtLeast32BitUnsigned
		+ Default
		+ Bounded
		+ Copy
		+ sp_std::hash::Hash
		+ sp_std::str::FromStr
		+ MaybeMallocSizeOf
		+ MaxEncodedLen
		+ TypeInfo;

	type Hash: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ SimpleBitOps
		+ Ord
		+ Default
		+ Copy
		+ CheckEqual
		+ sp_std::hash::Hash
		+ AsRef<[u8]>
		+ AsMut<[u8]>
		+ MaybeMallocSizeOf
		+ MaxEncodedLen;

	type Hashing: Hash<Output = Self::Hash> + TypeInfo;

	type AccountId: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ Ord
		+ MaxEncodedLen;

	type Lookup: StaticLookup<Target = Self::AccountId>;

	type Header: Parameter + traits::Header<Number = Self::BlockNumber, Hash = Self::Hash>;

	type RuntimeEvent: Parameter + Member + Debug;

	type BlockHashCount: Get<Self::BlockNumber>;

	type DbWeight;

	type Version;

	type AccountData: Member + FullCodec + Clone + Default + TypeInfo + MaxEncodedLen;

	type OnNewAccount: OnNewAccount<Self::AccountId>;

	type OnKilledAccount: OnKilledAccount<Self::AccountId>;

	type SystemWeightInfo;

	type SS58Prefix: Get<u16>;

	type OnSetCode;

	type MaxConsumers;
}

/// Simplifed pallet balances Config trait. Needed because substrate pallets compile to wasm
/// in no_std mode.
pub trait BalancesConfig: FrameSystemConfig {
	/// The balance of an account.
	type Balance: Parameter
		+ Member
		+ AtLeast32BitUnsigned
		+ Codec
		+ Default
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaxEncodedLen
		+ TypeInfo
		+ FixedPointOperand;

	/// Handler for the unbalanced reduction when removing a dust account.
	type DustRemoval;

	/// The overarching event type.
	type RuntimeEvent;

	/// The minimum amount required to keep an account open.
	type ExistentialDeposit: Get<Self::Balance>;

	/// The means of storing the balances of an account.
	type AccountStore: StoredMap<Self::AccountId, crate::AccountData<Self::Balance>>;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo;

	/// The maximum number of locks that should exist on an account.
	/// Not strictly enforced, but used for weight estimation.
	type MaxLocks: Get<u32>;

	/// The maximum number of named reserves that can exist on an account.
	type MaxReserves: Get<u32>;

	/// The id type for named reserves.
	type ReserveIdentifier: Parameter + Member + MaxEncodedLen + Ord + Copy;
}
