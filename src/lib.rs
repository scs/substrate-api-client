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
#![feature(assert_matches)]

#[cfg(feature = "std")]
pub mod std;

#[cfg(feature = "std")]
pub mod extrinsic;

pub mod utils;

#[cfg(feature = "std")]
pub use crate::std::*;

pub use ac_primitives::{
    AccountData, AccountDataGen, AccountInfo, AccountInfoGen, Balance, BlockNumber, GenericAddress,
    GenericExtra, Hash, Index, Moment, RefCount, UncheckedExtrinsicV4,
};

#[cfg(feature = "std")]
pub use ac_compose_macros::compose_extrinsic;

pub use ac_compose_macros::{compose_call, compose_extrinsic_offline};
