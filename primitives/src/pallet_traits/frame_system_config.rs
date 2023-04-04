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

use codec::{Codec, Decode, EncodeLike, FullCodec, MaxEncodedLen};
use core::fmt::Debug;
use scale_info::TypeInfo;
use serde::{de::DeserializeOwned, Serialize};
use sp_runtime::traits::{
	self, AtLeast32Bit, AtLeast32BitUnsigned, Bounded, CheckEqual, Dispatchable, Get, Hash, Member,
	SimpleBitOps, StaticLookup,
};

/// Simplified Frame system Config trait. Needed because substrate pallets compile to wasm
/// in no_std mode.
pub trait FrameSystemConfig {
	type BaseCallFilter;
	type BlockWeights;
	type BlockLength;
	type RuntimeOrigin: Clone;
	type RuntimeCall: Codec
		+ EncodeLike
		+ Clone
		+ Eq
		+ Debug
		+ TypeInfo
		+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
		+ Debug;
	/// This type enforces the (de)serialization implementation
	/// also in no-std mode (unlike substrates MaybeSerializeDeserialize).
	type Index: Serialize
		+ DeserializeOwned
		+ Debug
		+ Default
		+ AtLeast32Bit
		+ Copy
		+ MaxEncodedLen
		+ Decode;
	/// This type enforces the (de)serialization implementation
	/// also in no-std mode (unlike substrates MaybeSerializeDeserialize).
	type BlockNumber: Codec
		+ EncodeLike
		+ Clone
		+ Eq
		+ Debug
		+ TypeInfo
		+ Serialize
		+ DeserializeOwned
		+ Debug
		+ AtLeast32BitUnsigned
		+ Default
		+ Bounded
		+ Copy
		+ core::hash::Hash
		+ core::str::FromStr
		+ MaxEncodedLen
		+ TypeInfo;
	/// This type enforces the (de)serialization implementation
	/// also in no-std mode (unlike substrates MaybeSerializeDeserialize).
	/// A type redefinition might be necessary in no-std.
	/// See primitives/serde_impls for examples
	type Hash: Codec
		+ EncodeLike
		+ Clone
		+ Eq
		+ Debug
		+ TypeInfo
		+ Serialize
		+ DeserializeOwned
		+ Debug
		+ SimpleBitOps
		+ Ord
		+ Default
		+ Copy
		+ CheckEqual
		+ core::hash::Hash
		+ AsRef<[u8]>
		+ AsMut<[u8]>
		+ MaxEncodedLen;
	type Hashing: Hash<Output = Self::Hash> + TypeInfo;
	/// This type enforces the (de)serialization implementation
	/// also in no-std mode (unlike substrates MaybeSerializeDeserialize).
	/// A type redefinition might be necessary in no-std.
	/// See primitives/serde_impls for examples.
	type AccountId: Codec
		+ EncodeLike
		+ Clone
		+ Eq
		+ Debug
		+ TypeInfo
		+ Member
		+ Serialize
		+ DeserializeOwned
		+ Debug
		+ Ord
		+ MaxEncodedLen;
	type Lookup: StaticLookup<Target = Self::AccountId>;
	type Header: Codec
		+ EncodeLike
		+ Clone
		+ Eq
		+ Debug
		+ TypeInfo
		+ traits::Header<Number = Self::BlockNumber, Hash = Self::Hash>;
	type RuntimeEvent: Codec + EncodeLike + Clone + Eq + TypeInfo + Member + Debug;
	type BlockHashCount: Get<Self::BlockNumber>;
	type DbWeight;
	type Version;
	type AccountData: Member + FullCodec + Clone + Default + TypeInfo + MaxEncodedLen;
	type OnNewAccount;
	type OnKilledAccount;
	type SystemWeightInfo;
	type SS58Prefix: Get<u16>;
	type OnSetCode;
	type MaxConsumers;
}

#[cfg(feature = "std")]
impl<T> FrameSystemConfig for T
where
	T: frame_system::Config,
{
	type BaseCallFilter = T::BaseCallFilter;
	type BlockWeights = T::BlockWeights;
	type BlockLength = T::BlockLength;
	type RuntimeOrigin = T::RuntimeOrigin;
	type RuntimeCall = T::RuntimeCall;
	type Index = T::Index;
	type BlockNumber = T::BlockNumber;
	type Hash = T::Hash;
	type Hashing = T::Hashing;
	type AccountId = T::AccountId;
	type Lookup = T::Lookup;
	type Header = T::Header;
	type RuntimeEvent = T::RuntimeEvent;
	type BlockHashCount = T::BlockHashCount;
	type DbWeight = T::DbWeight;
	type Version = T::Version;
	type AccountData = T::AccountData;
	type OnNewAccount = T::OnNewAccount;
	type OnKilledAccount = T::OnKilledAccount;
	type SystemWeightInfo = T::SystemWeightInfo;
	type SS58Prefix = T::SS58Prefix;
	type OnSetCode = T::OnSetCode;
	type MaxConsumers = T::MaxConsumers;
}
