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
use sp_runtime::traits::Get;

/// Simplified pallet contract Config trait. Needed because substrate pallets compile to wasm
/// in no_std mode.
pub trait ContractsConfig: FrameSystemConfig {
	type Time;
	type Randomness;
	type Currency;
	type RuntimeEvent;
	type RuntimeCall: codec::Decode;
	type CallFilter;
	type WeightPrice;
	type WeightInfo;
	type ChainExtension: Default;
	type Schedule;
	type CallStack;
	type DepositPerByte;
	type DepositPerItem;
	type AddressGenerator;
	type MaxCodeLen: Get<u32>;
	type MaxStorageKeyLen: Get<u32>;
	type UnsafeUnstableInterface: Get<bool>;
	type MaxDebugBufferLen: Get<u32>;
}

#[cfg(feature = "contracts-xt")]
impl<T> ContractsConfig for T
where
	T: pallet_contracts::Config,
{
	type Time = T::Time;
	type Randomness = T::Randomness;
	type Currency = T::Currency;
	type RuntimeEvent = <T as pallet_contracts::Config>::RuntimeEvent;
	type RuntimeCall = <T as pallet_contracts::Config>::RuntimeCall;
	type CallFilter = T::CallFilter;
	type WeightPrice = T::WeightPrice;
	type WeightInfo = T::WeightInfo;
	type ChainExtension = T::ChainExtension;
	type Schedule = T::Schedule;
	type CallStack = T::CallStack;
	type DepositPerByte = T::DepositPerByte;
	type DepositPerItem = T::DepositPerItem;
	type AddressGenerator = T::AddressGenerator;
	type MaxCodeLen = T::MaxCodeLen;
	type MaxStorageKeyLen = T::MaxStorageKeyLen;
	type UnsafeUnstableInterface = T::UnsafeUnstableInterface;
	type MaxDebugBufferLen = T::MaxDebugBufferLen;
}
