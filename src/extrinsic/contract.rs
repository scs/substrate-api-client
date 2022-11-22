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

use crate::{api::Api, rpc::RpcClient, Hash, Index};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{Balance, CallIndex, ExtrinsicParams, GenericAddress, UncheckedExtrinsicV4};
use codec::Compact;
use sp_core::crypto::Pair;
use sp_runtime::{MultiSignature, MultiSigner};
use sp_std::prelude::*;

pub const CONTRACTS_MODULE: &str = "Contracts";
pub const CONTRACTS_PUT_CODE: &str = "put_code";
pub const CONTRACTS_INSTANTIATE: &str = "instantiate";
pub const CONTRACTS_INSTANTIATE_WITH_CODE: &str = "instantiate_with_code";
pub const CONTRACTS_CALL: &str = "call";

type Gas = u64;
type Data = Vec<u8>;
type Code = Vec<u8>;
type Salt = Vec<u8>;

type GasLimit = Compact<Gas>;
type Endowment = Compact<Balance>;
type Value = Compact<Balance>;
type Destination = GenericAddress;

pub type ContractPutCodeFn = (CallIndex, GasLimit, Data);
pub type ContractInstantiateFn = (CallIndex, Endowment, GasLimit, Hash, Data);
pub type ContractInstantiateWithCodeFn = (CallIndex, Endowment, GasLimit, Code, Data, Salt);
pub type ContractCallFn = (CallIndex, Destination, Value, GasLimit, Data);

pub type ContractPutCodeXt<SignedExtra> = UncheckedExtrinsicV4<ContractPutCodeFn, SignedExtra>;
pub type ContractInstantiateXt<SignedExtra> =
	UncheckedExtrinsicV4<ContractInstantiateFn, SignedExtra>;
pub type ContractInstantiateWithCodeXt<SignedExtra> =
	UncheckedExtrinsicV4<ContractInstantiateWithCodeFn, SignedExtra>;
pub type ContractCallXt<SignedExtra> = UncheckedExtrinsicV4<ContractCallFn, SignedExtra>;

#[cfg(feature = "std")]
impl<P, Client, Params> Api<P, Client, Params>
where
	P: Pair,
	MultiSignature: From<P::Signature>,
	MultiSigner: From<P::Public>,
	Client: RpcClient,
	Params: ExtrinsicParams<Index, Hash>,
{
	pub fn contract_put_code(
		&self,
		gas_limit: Gas,
		code: Data,
	) -> ContractPutCodeXt<Params::SignedExtra> {
		compose_extrinsic!(self, CONTRACTS_MODULE, CONTRACTS_PUT_CODE, Compact(gas_limit), code)
	}

	pub fn contract_instantiate(
		&self,
		endowment: Balance,
		gas_limit: Gas,
		code_hash: Hash,
		data: Data,
	) -> ContractInstantiateXt<Params::SignedExtra> {
		compose_extrinsic!(
			self,
			CONTRACTS_MODULE,
			CONTRACTS_INSTANTIATE,
			Compact(endowment),
			Compact(gas_limit),
			code_hash,
			data
		)
	}

	pub fn contract_instantiate_with_code(
		&self,
		endowment: Balance,
		gas_limit: Gas,
		code: Data,
		data: Data,
		salt: Data,
	) -> ContractInstantiateWithCodeXt<Params::SignedExtra> {
		compose_extrinsic!(
			self,
			CONTRACTS_MODULE,
			CONTRACTS_INSTANTIATE_WITH_CODE,
			Compact(endowment),
			Compact(gas_limit),
			code,
			data,
			salt
		)
	}

	pub fn contract_call(
		&self,
		dest: GenericAddress,
		value: Balance,
		gas_limit: Gas,
		data: Data,
	) -> ContractCallXt<Params::SignedExtra> {
		compose_extrinsic!(
			self,
			CONTRACTS_MODULE,
			CONTRACTS_CALL,
			dest,
			Compact(value),
			Compact(gas_limit),
			data
		)
	}
}
