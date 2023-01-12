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

use super::{AddressFor, AssignExtrinsicTypes, ExtrinsicFor};
use crate::{api::Api, rpc::Request};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{
	BalancesConfig, CallIndex, ContractsConfig, ExtrinsicParams, FrameSystemConfig, SignExtrinsic,
};
use alloc::vec::Vec;
use codec::{Compact, Encode};
use sp_runtime::traits::GetRuntimeBlockType;

pub const MODULE: &str = "Contracts";
pub const PUT_CODE: &str = "put_code";
pub const INSTANTIATE: &str = "instantiate";
pub const INSTANTIATE_WITH_CODE: &str = "instantiate_with_code";
pub const CALL: &str = "call";

pub type GasLimitFor<M> = Compact<<M as CreateContractsExtrinsic>::Gas>;
pub type ValueFor<M> = Compact<<M as CreateContractsExtrinsic>::Currency>;
pub type EndowmentFor<M> = Compact<<M as CreateContractsExtrinsic>::Currency>;
pub type DataFor<M> = <M as CreateContractsExtrinsic>::Data;
pub type CodeFor<M> = <M as CreateContractsExtrinsic>::Code;
pub type SaltFor<M> = <M as CreateContractsExtrinsic>::Salt;
pub type HashFor<M> = <M as CreateContractsExtrinsic>::Hash;

/// Call for putting code in a contract.
pub type PutCodeFor<M> = (CallIndex, GasLimitFor<M>, DataFor<M>);

/// Call for instantiating a contract with the code hash.
pub type InstantiateWithHashFor<M> =
	(CallIndex, EndowmentFor<M>, GasLimitFor<M>, HashFor<M>, DataFor<M>);

/// Call for instantiating a contract with code and salt.
pub type InstantiateWithCodeFor<M> =
	(CallIndex, EndowmentFor<M>, GasLimitFor<M>, CodeFor<M>, DataFor<M>, SaltFor<M>);

/// Call for calling a function inside a contract.
pub type ContractCallFor<M> = (CallIndex, AddressFor<M>, ValueFor<M>, GasLimitFor<M>, DataFor<M>);

pub trait CreateContractsExtrinsic: AssignExtrinsicTypes {
	type Gas;
	type Currency;
	type Hash;
	type Code;
	type Data;
	type Salt;

	fn contract_put_code(
		&self,
		gas_limit: Self::Gas,
		code: Self::Code,
	) -> ExtrinsicFor<Self, PutCodeFor<Self>>;

	fn contract_instantiate(
		&self,
		endowment: Self::Currency,
		gas_limit: Self::Gas,
		code_hash: Self::Hash,
		data: Self::Data,
	) -> ExtrinsicFor<Self, InstantiateWithHashFor<Self>>;

	fn contract_instantiate_with_code(
		&self,
		endowment: Self::Currency,
		gas_limit: Self::Gas,
		code: Self::Code,
		data: Self::Data,
		salt: Self::Salt,
	) -> ExtrinsicFor<Self, InstantiateWithCodeFor<Self>>;

	fn contract_call(
		&self,
		dest: Self::Address,
		value: Self::Currency,
		gas_limit: Self::Gas,
		data: Self::Data,
	) -> ExtrinsicFor<Self, ContractCallFor<Self>>;
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
{
	type Gas = u64;
	type Currency = BalanceOf<Runtime>;
	type Hash = Runtime::Hash;
	type Code = Vec<u8>;
	type Data = Vec<u8>;
	type Salt = Vec<u8>;

	fn contract_put_code(
		&self,
		gas_limit: Self::Gas,
		code: Self::Code,
	) -> ExtrinsicFor<Self, PutCodeFor<Self>> {
		compose_extrinsic!(self, MODULE, PUT_CODE, Compact(gas_limit), code)
	}

	fn contract_instantiate(
		&self,
		endowment: Self::Currency,
		gas_limit: Self::Gas,
		code_hash: Self::Hash,
		data: Self::Data,
	) -> ExtrinsicFor<Self, InstantiateWithHashFor<Self>> {
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
		code: Self::Code,
		data: Self::Data,
		salt: Self::Salt,
	) -> ExtrinsicFor<Self, InstantiateWithCodeFor<Self>> {
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
	) -> ExtrinsicFor<Self, ContractCallFor<Self>> {
		compose_extrinsic!(self, MODULE, CALL, dest, Compact(value), Compact(gas_limit), data)
	}
}
