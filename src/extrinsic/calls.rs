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

pub type GenericAddress = Address<[u8; 32], u32>;
pub type BalanceTransfer = ([u8; 2], GenericAddress, Compact<u128>);


#[macro_export]
macro_rules! compose_call {
    ( $ node_metadata: expr, $ module: expr, $ call_name: expr, $ ($args: expr), + ) => {
        {
            $node_metadata.retain(|m| !m.calls.is_empty());

            let module_index = $node_metadata
            .iter().position( | m | m.name == $module).unwrap();

            let call_index = $node_metadata[module_index].calls
            .iter().position( | c| c.name == $call_name).unwrap();

            ([module_index as u8, call_index as u8], $( ($args)), +)
        }
    };
}

pub fn balance_transfer_fn(to: GenericAddress, amount: u128, mut metadata: NodeMetadata) -> BalanceTransfer {
    compose_call!(metadata, BALANCES_MODULE_NAME, BALANCES_TRANSFER, to, Compact(amount))
}
