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
//! https://polkadot.js.org/docs/substrate/extrinsics/#contracts

// FIXME: This module is currently outdated. See https://github.com/scs/substrate-api-client/issues/435.

use crate::{api::Api, rpc::Request};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{
	CallIndex, ContractsConfig, ExtrinsicParams, FrameSystemConfig, SignExtrinsic,
	UncheckedExtrinsicV4,
};
use alloc::vec::Vec;
use codec::{Compact, Encode};

pub const CONTRACTS_MODULE: &str = "Contracts";
pub const PUT_CODE: &str = "put_code";
pub const INSTANTIATE: &str = "instantiate";
pub const INSTANTIATE_WITH_CODE: &str = "instantiate_with_code";
pub const CALL: &str = "call";

pub type GasLimitFor<M> = Compact<<M as ContractsExtrinsics>::Gas>;
pub type ValueFor<M> = Compact<<M as ContractsExtrinsics>::Currency>;
pub type EndowmentFor<M> = Compact<<M as ContractsExtrinsics>::Currency>;
pub type DataFor<M> = <M as ContractsExtrinsics>::Data;
pub type CodeFor<M> = <M as ContractsExtrinsics>::Code;
pub type SaltFor<M> = <M as ContractsExtrinsics>::Salt;
pub type HashFor<M> = <M as ContractsExtrinsics>::Hash;
pub type AddressFor<M> = <M as ContractsExtrinsics>::Address;

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

pub trait ContractsExtrinsics {
	type Gas;
	type Currency;
	type Hash;
	type Code;
	type Data;
	type Salt;
	type Address;
	type Extrinsic<Call>;

	fn contract_put_code(
		&self,
		gas_limit: Self::Gas,
		code: Self::Code,
	) -> Self::Extrinsic<PutCodeFor<Self>>;

	fn contract_instantiate(
		&self,
		endowment: Self::Currency,
		gas_limit: Self::Gas,
		code_hash: Self::Hash,
		data: Self::Data,
	) -> Self::Extrinsic<InstantiateWithHashFor<Self>>;

	fn contract_instantiate_with_code(
		&self,
		endowment: Self::Currency,
		gas_limit: Self::Gas,
		code: Self::Code,
		data: Self::Data,
		salt: Self::Salt,
	) -> Self::Extrinsic<InstantiateWithCodeFor<Self>>;

	fn contract_call(
		&self,
		dest: Self::Address,
		value: Self::Currency,
		gas_limit: Self::Gas,
		data: Self::Data,
	) -> Self::Extrinsic<ContractCallFor<Self>>;
}

#[cfg(feature = "std")]
type BalanceOf<T> = <<T as ContractsConfig>::Currency as frame_support::traits::Currency<
	<T as FrameSystemConfig>::AccountId,
>>::Balance;

#[cfg(feature = "std")]
impl<Signer, Client, Params, Runtime> ContractsExtrinsics for Api<Signer, Client, Params, Runtime>
where
	Signer: SignExtrinsic<Runtime::AccountId>,
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: ContractsConfig,
	Compact<BalanceOf<Runtime>>: Encode + Clone,
	Runtime::Currency: frame_support::traits::Currency<Runtime::AccountId>,
{
	type Gas = u64;
	type Currency = BalanceOf<Runtime>;
	type Hash = Runtime::Hash;
	type Code = Vec<u8>;
	type Data = Vec<u8>;
	type Salt = Vec<u8>;
	type Address = Signer::ExtrinsicAddress;
	type Extrinsic<Call> =
		UncheckedExtrinsicV4<Self::Address, Call, Signer::Signature, Params::SignedExtra>;

	fn contract_put_code(
		&self,
		gas_limit: Self::Gas,
		code: Self::Code,
	) -> Self::Extrinsic<PutCodeFor<Self>> {
		compose_extrinsic!(self, CONTRACTS_MODULE, PUT_CODE, Compact(gas_limit), code)
	}

	fn contract_instantiate(
		&self,
		endowment: Self::Currency,
		gas_limit: Self::Gas,
		code_hash: Self::Hash,
		data: Self::Data,
	) -> Self::Extrinsic<InstantiateWithHashFor<Self>> {
		compose_extrinsic!(
			self,
			CONTRACTS_MODULE,
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
	) -> Self::Extrinsic<InstantiateWithCodeFor<Self>> {
		compose_extrinsic!(
			self,
			CONTRACTS_MODULE,
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
	) -> Self::Extrinsic<ContractCallFor<Self>> {
		compose_extrinsic!(
			self,
			CONTRACTS_MODULE,
			CALL,
			dest,
			Compact(value),
			Compact(gas_limit),
			data
		)
	}
}
