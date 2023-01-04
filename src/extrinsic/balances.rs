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

use crate::{api::Api, rpc::Request};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{
	BalancesConfig, CallIndex, ExtrinsicParams, SignExtrinsic, UncheckedExtrinsicV4,
};
use alloc::borrow::ToOwned;
use codec::{Compact, Encode};
use serde::de::DeserializeOwned;
use sp_runtime::traits::GetRuntimeBlockType;

pub const BALANCES_MODULE: &str = "Balances";
pub const BALANCES_TRANSFER: &str = "transfer";
pub const BALANCES_SET_BALANCE: &str = "set_balance";

pub type BalanceTransferFn<Address, Balance> = (CallIndex, Address, Compact<Balance>);
pub type BalanceSetBalanceFn<Address, Balance> =
	(CallIndex, Address, Compact<Balance>, Compact<Balance>);

pub type BalanceTransferXt<Address, Balance, Signature, SignedExtra> =
	UncheckedExtrinsicV4<Address, BalanceTransferFn<Address, Balance>, Signature, SignedExtra>;
pub type BalanceSetBalanceXt<Address, Balance, Signature, SignedExtra> =
	UncheckedExtrinsicV4<Address, BalanceSetBalanceFn<Address, Balance>, Signature, SignedExtra>;

impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Signer: SignExtrinsic<Runtime::AccountId>,
	Client: Request,
	Runtime: GetRuntimeBlockType + BalancesConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Compact<Runtime::Balance>: Encode,
	Runtime::Header: DeserializeOwned,
	Runtime::RuntimeBlock: DeserializeOwned,
{
	pub fn balance_transfer(
		&self,
		to: Signer::ExtrinsicAddress,
		amount: Runtime::Balance,
	) -> BalanceTransferXt<
		Signer::ExtrinsicAddress,
		Runtime::Balance,
		Signer::Signature,
		Params::SignedExtra,
	> {
		compose_extrinsic!(self, BALANCES_MODULE, BALANCES_TRANSFER, to, Compact(amount))
	}

	pub fn balance_set_balance(
		&self,
		who: Signer::ExtrinsicAddress,
		free_balance: Runtime::Balance,
		reserved_balance: Runtime::Balance,
	) -> BalanceSetBalanceXt<
		Signer::ExtrinsicAddress,
		Runtime::Balance,
		Signer::Signature,
		Params::SignedExtra,
	> {
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
