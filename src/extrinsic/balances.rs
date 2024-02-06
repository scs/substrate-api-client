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
	config::Config, extrinsic_params::ExtrinsicParams, extrinsics::CallIndex, SignExtrinsic,
	UncheckedExtrinsicV4,
};
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use codec::{Compact, Encode};

pub const BALANCES_MODULE: &str = "Balances";
pub const TRANSFER_ALLOW_DEATH: &str = "transfer_allow_death";
pub const FORCE_SET_BALANCE: &str = "force_set_balance";

/// Call for a balance transfer.
pub type TransferAllowDeathCall<Address, Balance> = (CallIndex, Address, Compact<Balance>);

/// Call to the balance of an account.
pub type ForceSetBalanceCall<Address, Balance> = (CallIndex, Address, Compact<Balance>);

#[maybe_async::maybe_async(?Send)]
pub trait BalancesExtrinsics {
	type Balance;
	type Address;
	type Extrinsic<Call>;

	/// Transfer some liquid free balance to another account.
	#[allow(clippy::type_complexity)]
	async fn balance_transfer_allow_death(
		&self,
		to: Self::Address,
		amount: Self::Balance,
	) -> Option<Self::Extrinsic<TransferAllowDeathCall<Self::Address, Self::Balance>>>;

	///  Set the balances of a given account.
	#[allow(clippy::type_complexity)]
	async fn balance_force_set_balance(
		&self,
		who: Self::Address,
		free_balance: Self::Balance,
	) -> Option<Self::Extrinsic<ForceSetBalanceCall<Self::Address, Self::Balance>>>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> BalancesExtrinsics for Api<T, Client>
where
	T: Config,
	Client: Request,
	Compact<T::Balance>: Encode,
{
	type Balance = T::Balance;
	type Address = <T::ExtrinsicSigner as SignExtrinsic<T::AccountId>>::ExtrinsicAddress;
	type Extrinsic<Call> = UncheckedExtrinsicV4<
		Self::Address,
		Call,
		<T::ExtrinsicSigner as SignExtrinsic<T::AccountId>>::Signature,
		<T::ExtrinsicParams as ExtrinsicParams<T::Index, T::Hash>>::SignedExtra,
	>;

	async fn balance_transfer_allow_death(
		&self,
		to: Self::Address,
		amount: Self::Balance,
	) -> Option<Self::Extrinsic<TransferAllowDeathCall<Self::Address, Self::Balance>>> {
		compose_extrinsic!(self, BALANCES_MODULE, TRANSFER_ALLOW_DEATH, to, Compact(amount))
	}

	async fn balance_force_set_balance(
		&self,
		who: Self::Address,
		free_balance: Self::Balance,
	) -> Option<Self::Extrinsic<ForceSetBalanceCall<Self::Address, Self::Balance>>> {
		compose_extrinsic!(self, BALANCES_MODULE, FORCE_SET_BALANCE, who, Compact(free_balance))
	}
}
