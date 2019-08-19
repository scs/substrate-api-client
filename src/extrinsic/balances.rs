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

use codec::Compact;
use node_primitives::Hash;
use primitive_types::U256;

use crate::crypto::AccountKey;
use crate::node_metadata::NodeMetadata;
use crate::compose_extrinsic;

use super::xt_primitives::*;
pub const BALANCES_MODULE: &str = "Balances";
pub const BALANCES_TRANSFER: &str = "transfer";

pub type BalanceTransferFn = ([u8; 2], GenericAddress, Compact<u128>);

pub type BalanceTransferXt = UncheckedExtrinsicV3<BalanceTransferFn>;

pub fn transfer(from: AccountKey, to: GenericAddress, amount: u128, nonce: U256, genesis_hash: Hash, node_metadata: NodeMetadata) -> BalanceTransferXt {
    compose_extrinsic!(
		node_metadata,
		genesis_hash,
		BALANCES_MODULE,
		BALANCES_TRANSFER,
		GenericExtra::new(nonce.low_u32()),
		from,
		to,
		Compact(amount)
	)
}