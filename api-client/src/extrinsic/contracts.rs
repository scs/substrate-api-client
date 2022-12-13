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

use crate::{api::Api, rpc::Request};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{
	BalancesConfig, CallIndex, ContractsConfig, ExtrinsicParams, FrameSystemConfig, GenericAddress,
	UncheckedExtrinsicV4,
};
use codec::{Compact, Encode};
use serde::de::DeserializeOwned;
use sp_core::crypto::Pair;
use sp_runtime::{traits::GetRuntimeBlockType, MultiSignature, MultiSigner};
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
type Endowment<Currency> = Compact<Currency>;
type Value<Currency> = Compact<Currency>;
type Destination = GenericAddress;

pub type ContractPutCodeFn = (CallIndex, GasLimit, Data);
pub type ContractInstantiateFn<Currency, Hash> =
	(CallIndex, Endowment<Currency>, GasLimit, Hash, Data);
pub type ContractInstantiateWithCodeFn<Currency> =
	(CallIndex, Endowment<Currency>, GasLimit, Code, Data, Salt);
pub type ContractCallFn<Currency> = (CallIndex, Destination, Value<Currency>, GasLimit, Data);

pub type ContractPutCodeXt<SignedExtra> = UncheckedExtrinsicV4<ContractPutCodeFn, SignedExtra>;
pub type ContractInstantiateXt<SignedExtra, Currency, Hash> =
	UncheckedExtrinsicV4<ContractInstantiateFn<Currency, Hash>, SignedExtra>;
pub type ContractInstantiateWithCodeXt<SignedExtra, Currency> =
	UncheckedExtrinsicV4<ContractInstantiateWithCodeFn<Currency>, SignedExtra>;
pub type ContractCallXt<SignedExtra, Currency> =
	UncheckedExtrinsicV4<ContractCallFn<Currency>, SignedExtra>;

#[cfg(feature = "std")]
type BalanceOf<T> = <<T as ContractsConfig>::Currency as frame_support::traits::Currency<
	<T as FrameSystemConfig>::AccountId,
>>::Balance;

#[cfg(feature = "std")]
impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Signer: Pair,
	MultiSignature: From<Signer::Signature>,
	MultiSigner: From<Signer::Public>,
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: GetRuntimeBlockType + ContractsConfig + BalancesConfig,
	Compact<BalanceOf<Runtime>>: Encode + Clone,
	Runtime::AccountId: From<Signer::Public>,
	Runtime::Currency: frame_support::traits::Currency<Runtime::AccountId>,
	Runtime::Header: DeserializeOwned,
	Runtime::RuntimeBlock: DeserializeOwned,
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
		endowment: BalanceOf<Runtime>,
		gas_limit: Gas,
		code_hash: Runtime::Hash,
		data: Data,
	) -> ContractInstantiateXt<Params::SignedExtra, BalanceOf<Runtime>, Runtime::Hash> {
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
		endowment: BalanceOf<Runtime>,
		gas_limit: Gas,
		code: Data,
		data: Data,
		salt: Data,
	) -> ContractInstantiateWithCodeXt<Params::SignedExtra, BalanceOf<Runtime>> {
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
		value: BalanceOf<Runtime>,
		gas_limit: Gas,
		data: Data,
	) -> ContractCallXt<Params::SignedExtra, BalanceOf<Runtime>> {
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
