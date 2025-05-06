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

//! Offers some predefined extrinsics for common runtime modules.

pub use balances::BalancesExtrinsics;
#[cfg(feature = "contracts-xt")]
pub use contracts::ContractsExtrinsics;
#[cfg(feature = "staking-xt")]
pub use staking::StakingExtrinsics;
pub use utility::UtilityExtrinsics;

pub mod balances;
#[cfg(feature = "contracts-xt")]
pub mod contracts;
pub mod offline_extrinsic;
#[cfg(feature = "staking-xt")]
pub mod staking;
pub mod utility;
