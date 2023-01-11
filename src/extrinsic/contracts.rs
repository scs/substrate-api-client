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

//! Extrinsics for `pallet-contract`.
//! Contracts module is community maintained and not CI tested, therefore it may not work as is.

use super::{AssignExtrinsicTypes, ExtrinsicFor};
use crate::{api::Api, rpc::Request};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{
	BalancesConfig, CallIndex, ContractsConfig, ExtrinsicParams, FrameSystemConfig, SignExtrinsic,
};
use alloc::vec::Vec;
use codec::{Compact, Encode};
use serde::de::DeserializeOwned;
use sp_runtime::traits::GetRuntimeBlockType;

pub const MODULE: &str = "Contracts";
pub const PUT_CODE: &str = "put_code";
pub const INSTANTIATE: &str = "instantiate";
pub const INSTANTIATE_WITH_CODE: &str = "instantiate_with_code";
pub const CALL: &str = "call";

pub type PutCodeCall<Gas, Data> = (CallIndex, Compact<Gas>, Data);
pub type InstantiateCall<Currency, Gas, Hash, Data> =
	(CallIndex, Compact<Currency>, Compact<Gas>, Hash, Data);
pub type InstantiateWithCodeCall<Currency, Gas, Code, Data, Salt> =
	(CallIndex, Compact<Currency>, Compact<Gas>, Code, Data, Salt);
pub type CallCall<Address, Currency, Gas, Data> =
	(CallIndex, Address, Compact<Currency>, Compact<Gas>, Data);

pub trait CreateContractsExtrinsic: AssignExtrinsicTypes {
	type Gas;
	type Currency;
	type Hash;
	type Data;
	type Salt;

	fn contract_put_code(
		&self,
		gas_limit: Self::Gas,
		code: Self::Data,
	) -> ExtrinsicFor<Self, PutCodeCall<Self::Gas, Self::Data>>;

	fn contract_instantiate(
		&self,
		endowment: Self::Currency,
		gas_limit: Self::Gas,
		code_hash: Self::Hash,
		data: Self::Data,
	) -> ExtrinsicFor<Self, InstantiateCall<Self::Currency, Self::Gas, Self::Hash, Self::Data>>;

	fn contract_instantiate_with_code(
		&self,
		endowment: Self::Currency,
		gas_limit: Self::Gas,
		code: Self::Data,
		data: Self::Data,
		salt: Self::Salt,
	) -> ExtrinsicFor<
		Self,
		InstantiateWithCodeCall<Self::Currency, Self::Gas, Self::Data, Self::Data, Self::Salt>,
	>;

	fn contract_call(
		&self,
		dest: Self::Address,
		value: Self::Currency,
		gas_limit: Self::Gas,
		data: Self::Data,
	) -> ExtrinsicFor<Self, CallCall<Self::Address, Self::Currency, Self::Gas, Self::Data>>;
}

#[cfg(feature = "std")]
type BalanceOf<T> = <<T as ContractsConfig>::Currency as frame_support::traits::Currency<
	<T as FrameSystemConfig>::AccountId,
>>::Balance;

#[cfg(feature = "std")]
impl<Signer, Client, Params, Runtime> CreateContractsExtrinsic
	for Api<Signer, Client, Params, Runtime>
where
	Signer: SignExtrinsic<Runtime::AccountId>,
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: GetRuntimeBlockType + ContractsConfig + BalancesConfig,
	Compact<BalanceOf<Runtime>>: Encode + Clone,
	Runtime::Currency: frame_support::traits::Currency<Runtime::AccountId>,
	Runtime::Header: DeserializeOwned,
	Runtime::RuntimeBlock: DeserializeOwned,
{
	type Gas = u64;
	type Currency = BalanceOf<Runtime>;
	type Hash = Runtime::Hash;
	type Data = Vec<u8>;
	type Salt = Vec<u8>;

	fn contract_put_code(
		&self,
		gas_limit: Self::Gas,
		code: Self::Data,
	) -> ExtrinsicFor<Self, PutCodeCall<Self::Gas, Self::Data>> {
		compose_extrinsic!(self, MODULE, PUT_CODE, Compact(gas_limit), code)
	}

	fn contract_instantiate(
		&self,
		endowment: Self::Currency,
		gas_limit: Self::Gas,
		code_hash: Self::Hash,
		data: Self::Data,
	) -> ExtrinsicFor<Self, InstantiateCall<Self::Currency, Self::Gas, Self::Hash, Self::Data>> {
		compose_extrinsic!(
			self,
			MODULE,
			INSTANTIATE,
			Compact(endowment),
			Compact(gas_limit),
			code_hash,
			data
		)
	}

	fn contract_instantiate_with_code(
		&self,
		endowment: Self::Currency,
		gas_limit: Self::Gas,
		code: Self::Data,
		data: Self::Data,
		salt: Self::Salt,
	) -> ExtrinsicFor<
		Self,
		InstantiateWithCodeCall<Self::Currency, Self::Gas, Self::Data, Self::Data, Self::Salt>,
	> {
		compose_extrinsic!(
			self,
			MODULE,
			INSTANTIATE_WITH_CODE,
			Compact(endowment),
			Compact(gas_limit),
			code,
			data,
			salt
		)
	}

	fn contract_call(
		&self,
		dest: Self::Address,
		value: Self::Currency,
		gas_limit: Self::Gas,
		data: Self::Data,
	) -> ExtrinsicFor<Self, CallCall<Self::Address, Self::Currency, Self::Gas, Self::Data>> {
		compose_extrinsic!(self, MODULE, CALL, dest, Compact(value), Compact(gas_limit), data)
	}
}
