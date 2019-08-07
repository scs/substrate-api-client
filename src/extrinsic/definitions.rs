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

use indices::address::Address;
use node_primitives::{Index, Signature};
use parity_codec::Compact;
use runtime_primitives::generic::UncheckedMortalCompactExtrinsic;

pub const BALANCES_MODULE_NAME: &str = "balances";
pub const BALANCES_TRANSFER: &str = "transfer";

pub type GenericAddress = Address<[u8; 32], u32>;
pub type BalanceTransfer = ([u8; 2], GenericAddress, Compact<u128>);
pub type UncheckedExtrinsic<F> = UncheckedMortalCompactExtrinsic<GenericAddress, Index, F, Signature>;