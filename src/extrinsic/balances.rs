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

//! Extrinsics for `pallet-balances`.

use crate::{api::Api, rpc::RpcClient, Hash, Index};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{Balance, CallIndex, ExtrinsicParams, GenericAddress, UncheckedExtrinsicV4};
use codec::Compact;
use sp_core::crypto::Pair;
use sp_runtime::{MultiSignature, MultiSigner};

pub const BALANCES_MODULE: &str = "Balances";
pub const BALANCES_TRANSFER: &str = "transfer";
pub const BALANCES_SET_BALANCE: &str = "set_balance";

pub type BalanceTransferFn = (CallIndex, GenericAddress, Compact<Balance>);
pub type BalanceSetBalanceFn = (CallIndex, GenericAddress, Compact<Balance>, Compact<Balance>);

pub type BalanceTransferXt<SignedExtra> = UncheckedExtrinsicV4<BalanceTransferFn, SignedExtra>;
pub type BalanceSetBalanceXt<SignedExtra> = UncheckedExtrinsicV4<BalanceSetBalanceFn, SignedExtra>;

#[cfg(feature = "std")]
impl<P, Client, Params> Api<P, Client, Params>
where
	P: Pair,
	MultiSignature: From<P::Signature>,
	MultiSigner: From<P::Public>,
	Client: RpcClient,
	Params: ExtrinsicParams<Index, Hash>,
{
	pub fn balance_transfer(
		&self,
		to: GenericAddress,
		amount: Balance,
	) -> BalanceTransferXt<Params::SignedExtra> {
		compose_extrinsic!(self, BALANCES_MODULE, BALANCES_TRANSFER, to, Compact(amount))
	}

	pub fn balance_set_balance(
		&self,
		who: GenericAddress,
		free_balance: Balance,
		reserved_balance: Balance,
	) -> BalanceSetBalanceXt<Params::SignedExtra> {
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
