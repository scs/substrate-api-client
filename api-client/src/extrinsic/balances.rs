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
	BalancesConfig, CallIndex, ExtrinsicParams, GenericAddress, UncheckedExtrinsicV4,
};
use codec::{Compact, Encode};
use serde::de::DeserializeOwned;
use sp_core::crypto::Pair;
use sp_runtime::{traits::GetRuntimeBlockType, MultiSignature, MultiSigner};

pub const BALANCES_MODULE: &str = "Balances";
pub const BALANCES_TRANSFER: &str = "transfer";
pub const BALANCES_SET_BALANCE: &str = "set_balance";

pub type BalanceTransferFn<Balance> = (CallIndex, GenericAddress, Compact<Balance>);
pub type BalanceSetBalanceFn<Balance> =
	(CallIndex, GenericAddress, Compact<Balance>, Compact<Balance>);

pub type BalanceTransferXt<SignedExtra, Balance> =
	UncheckedExtrinsicV4<BalanceTransferFn<Balance>, SignedExtra>;
pub type BalanceSetBalanceXt<SignedExtra, Balance> =
	UncheckedExtrinsicV4<BalanceSetBalanceFn<Balance>, SignedExtra>;

#[cfg(feature = "std")]
impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Signer: Pair,
	MultiSignature: From<Signer::Signature>,
	MultiSigner: From<Signer::Public>,
	Client: Request,
	Runtime: GetRuntimeBlockType + BalancesConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Compact<Runtime::Balance>: Encode,
	Runtime::Header: DeserializeOwned,
	Runtime::RuntimeBlock: DeserializeOwned,
	Runtime::AccountId: From<Signer::Public>,
{
	pub fn balance_transfer(
		&self,
		to: GenericAddress,
		amount: Runtime::Balance,
	) -> BalanceTransferXt<Params::SignedExtra, Runtime::Balance> {
		compose_extrinsic!(self, BALANCES_MODULE, BALANCES_TRANSFER, to, Compact(amount))
	}

	pub fn balance_set_balance(
		&self,
		who: GenericAddress,
		free_balance: Runtime::Balance,
		reserved_balance: Runtime::Balance,
	) -> BalanceSetBalanceXt<Params::SignedExtra, Runtime::Balance> {
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
