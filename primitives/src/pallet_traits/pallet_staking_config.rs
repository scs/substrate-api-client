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
use crate::FrameSystemConfig;
use codec::MaxEncodedLen;
use scale_info::TypeInfo;
use serde::{de::DeserializeOwned, Serialize};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Get},
	Perbill,
};
use sp_staking::{EraIndex, SessionIndex};

/// The balance type of this pallet.
pub type BalanceOf<T> = <T as StakingConfig>::CurrencyBalance;

/// Simplified pallet staking Config trait. Needed because substrate pallets compile to wasm
/// in no_std mode.
pub trait StakingConfig: FrameSystemConfig {
	type Currency;
	/// This type enforces the (de)serialization implementation
	/// also in no-std mode (unlike substrates MaybeSerializeDeserialize).
	type CurrencyBalance: AtLeast32BitUnsigned
		+ codec::FullCodec
		+ Copy
		+ Serialize
		+ DeserializeOwned
		+ core::fmt::Debug
		+ Default
		+ From<u64>
		+ TypeInfo
		+ MaxEncodedLen;
	type UnixTime;
	type CurrencyToVote;
	type ElectionProvider;
	type GenesisElectionProvider;
	type MaxNominations: Get<u32>;
	type HistoryDepth: Get<u32>;
	type RewardRemainder;
	type RuntimeEvent;
	type Slash;
	type Reward;
	type SessionsPerEra: Get<SessionIndex>;
	type BondingDuration: Get<EraIndex>;
	type SlashDeferDuration: Get<EraIndex>;
	type AdminOrigin;
	type SessionInterface;
	type EraPayout;
	type NextNewSession;
	type MaxNominatorRewardedPerValidator: Get<u32>;
	type OffendingValidatorsThreshold: Get<Perbill>;
	type VoterList;
	type TargetList;
	type MaxUnlockingChunks: Get<u32>;
	type OnStakerSlash: sp_staking::OnStakerSlash<Self::AccountId, BalanceOf<Self>>;
	type WeightInfo;
}

#[cfg(feature = "staking-xt")]
impl<T> StakingConfig for T
where
	T: pallet_staking::Config,
{
	type Currency = T::Currency;
	type CurrencyBalance = T::CurrencyBalance;
	type UnixTime = T::UnixTime;
	type CurrencyToVote = T::CurrencyToVote;
	type ElectionProvider = T::ElectionProvider;
	type GenesisElectionProvider = T::GenesisElectionProvider;
	type MaxNominations = T::MaxNominations;
	type HistoryDepth = T::HistoryDepth;
	type RewardRemainder = T::RewardRemainder;
	type RuntimeEvent = <T as pallet_staking::Config>::RuntimeEvent;
	type Slash = T::Slash;
	type Reward = T::Reward;
	type SessionsPerEra = T::SessionsPerEra;
	type BondingDuration = T::BondingDuration;
	type SlashDeferDuration = T::SlashDeferDuration;
	type AdminOrigin = T::AdminOrigin;
	type SessionInterface = T::SessionInterface;
	type EraPayout = T::EraPayout;
	type NextNewSession = T::NextNewSession;
	type MaxNominatorRewardedPerValidator = T::MaxNominatorRewardedPerValidator;
	type OffendingValidatorsThreshold = T::OffendingValidatorsThreshold;
	type VoterList = T::VoterList;
	type TargetList = T::TargetList;
	type MaxUnlockingChunks = T::MaxUnlockingChunks;
	type OnStakerSlash = T::OnStakerSlash;
	type WeightInfo = T::WeightInfo;
}
