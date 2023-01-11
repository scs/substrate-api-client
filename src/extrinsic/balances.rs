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

pub const MODULE: &str = "Balances";
pub const TRANSFER: &str = "transfer";
pub const SET_BALANCE: &str = "set_balance";

pub type BalanceTransferCall<Address, Balance> = (CallIndex, Address, Compact<Balance>);
pub type BalanceSetBalanceCall<Address, Balance> =
	(CallIndex, Address, Compact<Balance>, Compact<Balance>);

type AddressFor<Module> = <Module as CreateBalancesExtrinsic>::Address;
type SignatureFor<Module> = <Module as CreateBalancesExtrinsic>::Signature;
type SignedExtraFor<Module> = <Module as CreateBalancesExtrinsic>::SignedExtra;

type ExtrinsicFor<Module, Call> =
	UncheckedExtrinsicV4<AddressFor<Module>, Call, SignatureFor<Module>, SignedExtraFor<Module>>;

pub trait CreateBalancesExtrinsic {
	type Balance;
	type Address;
	type Signature;
	type SignedExtra;

	fn balance_transfer(
		&self,
		to: Self::Address,
		amount: Self::Balance,
	) -> ExtrinsicFor<Self, BalanceTransferCall<Self::Address, Self::Balance>>;

	fn balance_set_balance(
		&self,
		who: Self::Address,
		free_balance: Self::Balance,
		reserved_balance: Self::Balance,
	) -> ExtrinsicFor<Self, BalanceSetBalanceCall<Self::Address, Self::Balance>>;
}

impl<Signer, Client, Params, Runtime> CreateBalancesExtrinsic
	for Api<Signer, Client, Params, Runtime>
where
	Signer: SignExtrinsic<Runtime::AccountId>,
	Client: Request,
	Runtime: BalancesConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Compact<Runtime::Balance>: Encode,
	Runtime::Header: DeserializeOwned,
{
	type Balance = Runtime::Balance;
	type Address = Signer::ExtrinsicAddress;
	type Signature = Signer::Signature;
	type SignedExtra = Params::SignedExtra;

	fn balance_transfer(
		&self,
		to: Self::Address,
		amount: Self::Balance,
	) -> ExtrinsicFor<Self, BalanceTransferCall<Self::Address, Self::Balance>> {
		compose_extrinsic!(self, MODULE, TRANSFER, to, Compact(amount))
	}

	fn balance_set_balance(
		&self,
		who: Self::Address,
		free_balance: Self::Balance,
		reserved_balance: Self::Balance,
	) -> ExtrinsicFor<Self, BalanceSetBalanceCall<Self::Address, Self::Balance>> {
		compose_extrinsic!(
			self,
			MODULE,
			SET_BALANCE,
			who,
			Compact(free_balance),
			Compact(reserved_balance)
		)
	}
}
