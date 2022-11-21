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

pub mod extrinsic_params;
pub mod extrinsics;

/// The block number type used in this runtime.
pub type BlockNumber = u64;
/// The timestamp moment type used in this runtime.
pub type Moment = u64;
/// Index of a transaction.
pub type Index = u32;

pub type Hash = sp_core::H256;

pub type Balance = u128;

pub use frame_system::AccountInfo as GenericAccountInfo;
pub use pallet_balances::AccountData as GenericAccountData;

pub type AccountData = GenericAccountData<Balance>;
pub type AccountInfo = GenericAccountInfo<Index, AccountData>;
