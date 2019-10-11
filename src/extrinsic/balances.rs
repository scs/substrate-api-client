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


#[cfg(feature = "std")]
use crate::{Api, compose_extrinsic};
use primitives::crypto::Pair;
use codec::Encode;
use super::xt_primitives::*;

pub const BALANCES_MODULE: &str = "Balances";
pub const BALANCES_TRANSFER: &str = "transfer";
pub const BALANCES_SET_BALANCE: &str = "set_balance";

pub type BalanceTransferFn<P> = ([u8; 2], P, Compact<u128>);
pub type BalanceSetBalanceFn<P> = ([u8; 2], P, Compact<u128>, Compact<u128>);

pub type BalanceTransferXt<Pair, Public> = UncheckedExtrinsicV3<BalanceTransferFn<Public>, Pair>;
pub type BalanceSetBalanceXt<Pair, Public> = UncheckedExtrinsicV3<BalanceSetBalanceFn<Public>, Pair>;

#[cfg(feature = "std")]
impl<P> Api<P>
    where
        P: Pair,
        P::Public: Encode,
{
    pub fn transfer(&self, to: P::Public, amount: u128) -> BalanceTransferXt<P, P::Public> {
            compose_extrinsic!(
            self,
            BALANCES_MODULE,
            BALANCES_TRANSFER,
            to,
            Compact(amount)
        )
    }

    pub fn set_balance(&self, who: P::Public, free_balance: u128, reserved_balance: u128) -> BalanceSetBalanceXt<P, P::Public> {
            compose_extrinsic!(
            self,
            BALANCES_MODULE,
            BALANCES_SET_BALANCE,
            who,
            Compact(free_balance),
            Compact(reserved_balance)
        )
    }
}
