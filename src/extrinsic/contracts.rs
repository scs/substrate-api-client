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
	BalancesConfig, CallIndex, ContractsConfig, ExtrinsicParams, FrameSystemConfig, SignExtrinsic,
	UncheckedExtrinsicV4,
};
use alloc::vec::Vec;
use codec::{Compact, Encode};
use serde::de::DeserializeOwned;
use sp_runtime::traits::GetRuntimeBlockType;

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

pub type ContractPutCodeFn = (CallIndex, GasLimit, Data);
pub type ContractInstantiateFn<Currency, Hash> =
	(CallIndex, Endowment<Currency>, GasLimit, Hash, Data);
pub type ContractInstantiateWithCodeFn<Currency> =
	(CallIndex, Endowment<Currency>, GasLimit, Code, Data, Salt);
pub type ContractCallFn<Address, Currency> = (CallIndex, Address, Value<Currency>, GasLimit, Data);

pub type ContractPutCodeXt<Address, Signature, SignedExtra> =
	UncheckedExtrinsicV4<Address, ContractPutCodeFn, Signature, SignedExtra>;
pub type ContractInstantiateXt<Address, Signature, SignedExtra, Currency, Hash> =
	UncheckedExtrinsicV4<Address, ContractInstantiateFn<Currency, Hash>, Signature, SignedExtra>;
pub type ContractInstantiateWithCodeXt<Address, Signature, SignedExtra, Currency> =
	UncheckedExtrinsicV4<Address, ContractInstantiateWithCodeFn<Currency>, Signature, SignedExtra>;
pub type ContractCallXt<Address, Signature, SignedExtra, Currency> =
	UncheckedExtrinsicV4<Address, ContractCallFn<Address, Currency>, Signature, SignedExtra>;

#[cfg(feature = "std")]
type BalanceOf<T> = <<T as ContractsConfig>::Currency as frame_support::traits::Currency<
	<T as FrameSystemConfig>::AccountId,
>>::Balance;
type ExtrinsicAddressOf<Signer, AccountId> = <Signer as SignExtrinsic<AccountId>>::ExtrinsicAddress;
type SignatureOf<Signer, AccountId> = <Signer as SignExtrinsic<AccountId>>::Signature;
type HashOf<Runtime> = <Runtime as FrameSystemConfig>::Hash;
type AccountIdOf<Runtime> = <Runtime as FrameSystemConfig>::AccountId;

#[cfg(feature = "std")]
type ContractInstantiateXtOf<Signer, SignedExtra, Runtime> = ContractInstantiateXt<
	ExtrinsicAddressOf<Signer, AccountIdOf<Runtime>>,
	SignatureOf<Signer, AccountIdOf<Runtime>>,
	SignedExtra,
	BalanceOf<Runtime>,
	HashOf<Runtime>,
>;

#[cfg(feature = "std")]
impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
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
	pub fn contract_put_code(
		&self,
		gas_limit: Gas,
		code: Data,
	) -> ContractPutCodeXt<Signer::ExtrinsicAddress, Signer::Signature, Params::SignedExtra> {
		compose_extrinsic!(self, CONTRACTS_MODULE, CONTRACTS_PUT_CODE, Compact(gas_limit), code)
	}

	pub fn contract_instantiate(
		&self,
		endowment: BalanceOf<Runtime>,
		gas_limit: Gas,
		code_hash: Runtime::Hash,
		data: Data,
	) -> ContractInstantiateXtOf<Signer, Params::SignedExtra, Runtime> {
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
	) -> ContractInstantiateWithCodeXt<
		Signer::ExtrinsicAddress,
		Signer::Signature,
		Params::SignedExtra,
		BalanceOf<Runtime>,
	> {
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
		dest: Signer::ExtrinsicAddress,
		value: BalanceOf<Runtime>,
		gas_limit: Gas,
		data: Data,
	) -> ContractCallXt<
		Signer::ExtrinsicAddress,
		Signer::Signature,
		Params::SignedExtra,
		BalanceOf<Runtime>,
	> {
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
