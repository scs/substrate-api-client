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
//! https://polkadot.js.org/docs/substrate/extrinsics/#balances

use crate::{api::Api, rpc::Request};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{
	BalancesConfig, CallIndex, ExtrinsicParams, SignExtrinsic, UncheckedExtrinsicV4,
};
use alloc::borrow::ToOwned;
use codec::{Compact, Encode};

pub const BALANCES_MODULE: &str = "Balances";
pub const TRANSFER: &str = "transfer";
pub const SET_BALANCE: &str = "set_balance";

/// Call for a balance transfer.
pub type TransferCall<Address, Balance> = (CallIndex, Address, Compact<Balance>);

/// Call to the balance of an account.
pub type SetBalanceCall<Address, Balance> =
	(CallIndex, Address, Compact<Balance>, Compact<Balance>);

pub trait BalancesExtrinsics {
	type Balance;
	type Address;
	type Extrinsic<Call>;

	/// Transfer some liquid free balance to another account.
	fn balance_transfer(
		&self,
		to: Self::Address,
		amount: Self::Balance,
	) -> Self::Extrinsic<TransferCall<Self::Address, Self::Balance>>;

	///  Set the balances of a given account.
	fn balance_set_balance(
		&self,
		who: Self::Address,
		free_balance: Self::Balance,
		reserved_balance: Self::Balance,
	) -> Self::Extrinsic<SetBalanceCall<Self::Address, Self::Balance>>;
}

impl<Signer, Client, Params, Runtime> BalancesExtrinsics for Api<Signer, Client, Params, Runtime>
where
	Signer: SignExtrinsic<Runtime::AccountId>,
	Client: Request,
	Runtime: BalancesConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Compact<Runtime::Balance>: Encode,
{
	type Balance = Runtime::Balance;
	type Address = Signer::ExtrinsicAddress;
	type Extrinsic<Call> =
		UncheckedExtrinsicV4<Self::Address, Call, Signer::Signature, Params::SignedExtra>;

	fn balance_transfer(
		&self,
		to: Self::Address,
		amount: Self::Balance,
	) -> Self::Extrinsic<TransferCall<Self::Address, Self::Balance>> {
		compose_extrinsic!(self, BALANCES_MODULE, TRANSFER, to, Compact(amount))
	}

	fn balance_set_balance(
		&self,
		who: Self::Address,
		free_balance: Self::Balance,
		reserved_balance: Self::Balance,
	) -> Self::Extrinsic<SetBalanceCall<Self::Address, Self::Balance>> {
		compose_extrinsic!(
			self,
			BALANCES_MODULE,
			SET_BALANCE,
			who,
			Compact(free_balance),
			Compact(reserved_balance)
		)
	}
}
