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
use parity_codec::Compact;
use crate::node_metadata::NodeMetadata;

pub const BALANCES_MODULE_NAME: &str = "balances";
pub const BALANCES_TRANSFER: &str = "transfer";

pub type BalanceTransfer = ([u8; 2], Address::<[u8; 32], u32>, Compact<u128>);

pub fn balance_transfer_fn(to: Address::<[u8; 32], u32>, amount: u128, mut metadata: NodeMetadata) -> BalanceTransfer {
    metadata.retain(|m| !m.calls.is_empty());

    let balance_module_index = metadata
    .iter().position(|m| m.name == BALANCES_MODULE_NAME)
        .unwrap();

    let balance_transfer_index = metadata[balance_module_index].calls
        .iter().position(|c| c.name == BALANCES_TRANSFER)
        .unwrap();

    ([balance_module_index as u8, balance_transfer_index as u8], to, Compact(amount))
}
