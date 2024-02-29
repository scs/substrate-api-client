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

use crate::{api::Api, rpc::Request};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{
	config::Config, extrinsic_params::ExtrinsicParams, extrinsics::CallIndex, Determinism,
	SignExtrinsic, UncheckedExtrinsicV4, Weight,
};
use codec::{Compact, Encode};
use sp_core::Bytes;

pub const CONTRACTS_MODULE: &str = "Contracts";
pub const UPLOAD_CODE: &str = "upload_code";
pub const REMOVE_CODE: &str = "remove_code";
pub const SET_CODE: &str = "set_code";
pub const CALL: &str = "call";
pub const INSTANTIATE_WITH_CODE: &str = "instantiate_with_code";
pub const INSTANTIATE: &str = "instantiate";
pub const MIGRATE: &str = "migrate";

/// Upload new `code` without instantiating a contract from it..
pub type UploadCodeCall<P> = (CallIndex, CodeFor<P>, Option<CurrencyFor<P>>, DeterminismFor<P>);

/// Remove the code stored under `code_hash` and refund the deposit to its owner.
pub type RemoveCodeCall<P> = (CallIndex, CodeHashFor<P>);

/// Privileged function that changes the code of an existing contract.
pub type SetCodeCall<P> = (CallIndex, AddressFor<P>, CodeHashFor<P>);

/// Makes a call to an account, optionally transferring some balance.
pub type ContractCall<P> =
	(CallIndex, AddressFor<P>, CurrencyFor<P>, WeightFor<P>, Option<CurrencyFor<P>>, DataFor<P>);

/// Instantiates a new contract from the supplied `code` optionally transferring
/// some balance.
pub type InstantiateWithCodeCall<P> = (
	CallIndex,
	CurrencyFor<P>,
	WeightFor<P>,
	Option<CurrencyFor<P>>,
	CodeFor<P>,
	DataFor<P>,
	SaltFor<P>,
);

/// Instantiates a contract from a previously deployed wasm binary.
pub type InstantiateCall<P> = (
	CallIndex,
	CurrencyFor<P>,
	WeightFor<P>,
	Option<CurrencyFor<P>>,
	CodeHashFor<P>,
	DataFor<P>,
	SaltFor<P>,
);

/// Calls that contribute to advancing the migration have their fees waived, as it's helpful
/// for the chain.
pub type MigrateCall<P> = (CallIndex, WeightFor<P>);

pub type WeightFor<P> = <P as ContractsExtrinsics>::Weight;
pub type DeterminismFor<P> = <P as ContractsExtrinsics>::Determinism;
pub type DataFor<P> = <P as ContractsExtrinsics>::Data;
pub type CodeFor<P> = <P as ContractsExtrinsics>::Code;
pub type SaltFor<P> = <P as ContractsExtrinsics>::Salt;
pub type CodeHashFor<P> = <P as ContractsExtrinsics>::CodeHash;
pub type AddressFor<P> = <P as ContractsExtrinsics>::Address;
pub type CurrencyFor<P> = Compact<<P as ContractsExtrinsics>::Currency>;
#[maybe_async::maybe_async(?Send)]
pub trait ContractsExtrinsics {
	type Weight;
	type Currency;
	type Determinism;
	type CodeHash;
	type Code;
	type Data;
	type Salt;
	type Address;
	type Extrinsic<Call>;

	/// Upload new `code` without instantiating a contract from it.
	///
	/// If the code does not already exist a deposit is reserved from the caller
	/// and unreserved only when [`Self::remove_code`] is called. The size of the reserve
	/// depends on the size of the supplied `code`.
	///
	/// If the code already exists in storage it will still return `Ok` and upgrades
	/// the in storage version to the current
	/// [`InstructionWeights::version`](InstructionWeights).
	///
	/// - `determinism`: If this is set to any other value but [`Determinism::Enforced`] then
	///   the only way to use this code is to delegate call into it from an offchain execution.
	///   Set to [`Determinism::Enforced`] if in doubt.
	///
	/// # Note
	///
	/// Anyone can instantiate a contract from any uploaded code and thus prevent its removal.
	/// To avoid this situation a constructor could employ access control so that it can
	/// only be instantiated by permissioned entities. The same is true when uploading
	/// through [`Self::instantiate_with_code`].
	async fn contract_upload_code(
		&self,
		code: Self::Code,
		storage_deposit_limit: Option<Self::Currency>,
		determinism: Self::Determinism,
	) -> Option<Self::Extrinsic<UploadCodeCall<Self>>>;

	/// Remove the code stored under `code_hash` and refund the deposit to its owner.
	///
	/// A code can only be removed by its original uploader (its owner) and only if it is
	/// not used by any contract.
	async fn contract_remove_code(
		&self,
		code_hash: Self::CodeHash,
	) -> Option<Self::Extrinsic<RemoveCodeCall<Self>>>;

	/// Privileged function that changes the code of an existing contract.
	///
	/// This takes care of updating refcounts and all other necessary operations. Returns
	/// an error if either the `code_hash` or `dest` do not exist.
	///
	/// # Note
	///
	/// This does **not** change the address of the contract in question. This means
	/// that the contract address is no longer derived from its code hash after calling
	/// this dispatchable.
	async fn contract_set_code(
		&self,
		dest: Self::Address,
		code_hash: Self::CodeHash,
	) -> Option<Self::Extrinsic<SetCodeCall<Self>>>;

	/// Makes a call to an account, optionally transferring some balance.
	///
	/// # Parameters
	///
	/// * `dest`: Address of the contract to call.
	/// * `value`: The balance to transfer from the `origin` to `dest`.
	/// * `gas_limit`: The gas limit enforced when executing the constructor.
	/// * `storage_deposit_limit`: The maximum amount of balance that can be charged from the
	///   caller to pay for the storage consumed.
	/// * `data`: The input data to pass to the contract.
	///
	/// * If the account is a smart-contract account, the associated code will be
	/// executed and any value will be transferred.
	/// * If the account is a regular account, any value will be transferred.
	/// * If no account exists and the call value is not less than `existential_deposit`,
	/// a regular account will be created and any value will be transferred.
	async fn contract_call(
		&self,
		dest: Self::Address,
		value: Self::Currency,
		gas_limit: Self::Weight,
		storage_deposit_limit: Option<Self::Currency>,
		data: Self::Data,
	) -> Option<Self::Extrinsic<ContractCall<Self>>>;

	/// Instantiates a new contract from the supplied `code` optionally transferring
	/// some balance.
	///
	/// This dispatchable has the same effect as calling [`Self::upload_code`] +
	/// [`Self::instantiate`]. Bundling them together provides efficiency gains. Please
	/// also check the documentation of [`Self::upload_code`].
	///
	/// # Parameters
	///
	/// * `value`: The balance to transfer from the `origin` to the newly created contract.
	/// * `gas_limit`: The gas limit enforced when executing the constructor.
	/// * `storage_deposit_limit`: The maximum amount of balance that can be charged/reserved
	///   from the caller to pay for the storage consumed.
	/// * `code`: The contract code to deploy in raw bytes.
	/// * `data`: The input data to pass to the contract constructor.
	/// * `salt`: Used for the address derivation. See [`Pallet::contract_address`].
	///
	/// Instantiation is executed as follows:
	///
	/// - The supplied `code` is deployed, and a `code_hash` is created for that code.
	/// - If the `code_hash` already exists on the chain the underlying `code` will be shared.
	/// - The destination address is computed based on the sender, code_hash and the salt.
	/// - The smart-contract account is created at the computed address.
	/// - The `value` is transferred to the new account.
	/// - The `deploy` function is executed in the context of the newly-created account.
	async fn contract_instantiate_with_code(
		&self,
		value: Self::Currency,
		gas_limit: Self::Weight,
		storage_deposit_limit: Option<Self::Currency>,
		code: Self::Code,
		data: Self::Data,
		salt: Self::Salt,
	) -> Option<Self::Extrinsic<InstantiateWithCodeCall<Self>>>;

	/// Instantiates a contract from a previously deployed wasm binary.
	///
	/// This function is identical to [`Self::instantiate_with_code`] but without the
	/// code deployment step. Instead, the `code_hash` of an on-chain deployed wasm binary
	/// must be supplied.
	async fn contract_instantiate(
		&self,
		value: Self::Currency,
		gas_limit: Self::Weight,
		storage_deposit_limit: Option<Self::Currency>,
		code_hash: Self::CodeHash,
		data: Self::Data,
		salt: Self::Salt,
	) -> Option<Self::Extrinsic<InstantiateCall<Self>>>;

	/// When a migration is in progress, this dispatchable can be used to run migration steps.
	/// Calls that contribute to advancing the migration have their fees waived, as it's helpful
	/// for the chain. Note that while the migration is in progress, the pallet will also
	/// leverage the `on_idle` hooks to run migration steps.
	async fn contract_migrate(
		&self,
		weight_limit: Self::Weight,
	) -> Option<Self::Extrinsic<MigrateCall<Self>>>;
}

#[cfg(feature = "std")]
#[maybe_async::maybe_async(?Send)]
impl<T, Client> ContractsExtrinsics for Api<T, Client>
where
	T: Config,
	Client: Request,
	Compact<T::ContractCurrency>: Encode + Clone,
{
	type Weight = Weight;
	type Currency = T::ContractCurrency;
	type Determinism = Determinism;
	type CodeHash = T::Hash;
	type Code = Bytes;
	type Data = Bytes;
	type Salt = Bytes;
	type Address = <T::ExtrinsicSigner as SignExtrinsic<T::AccountId>>::ExtrinsicAddress;
	type Extrinsic<Call> = UncheckedExtrinsicV4<
		Self::Address,
		Call,
		<T::ExtrinsicSigner as SignExtrinsic<T::AccountId>>::Signature,
		<T::ExtrinsicParams as ExtrinsicParams<T::Index, T::Hash>>::SignedExtra,
	>;

	async fn contract_upload_code(
		&self,
		code: Self::Code,
		storage_deposit_limit: Option<Self::Currency>,
		determinism: Self::Determinism,
	) -> Option<Self::Extrinsic<UploadCodeCall<Self>>> {
		compose_extrinsic!(
			self,
			CONTRACTS_MODULE,
			UPLOAD_CODE,
			code,
			storage_deposit_limit.map(Compact),
			determinism
		)
	}

	async fn contract_remove_code(
		&self,
		code_hash: Self::CodeHash,
	) -> Option<Self::Extrinsic<RemoveCodeCall<Self>>> {
		compose_extrinsic!(self, CONTRACTS_MODULE, REMOVE_CODE, code_hash)
	}

	async fn contract_set_code(
		&self,
		dest: Self::Address,
		code_hash: Self::CodeHash,
	) -> Option<Self::Extrinsic<SetCodeCall<Self>>> {
		compose_extrinsic!(self, CONTRACTS_MODULE, SET_CODE, dest, code_hash)
	}

	async fn contract_call(
		&self,
		dest: Self::Address,
		value: Self::Currency,
		gas_limit: Self::Weight,
		storage_deposit_limit: Option<Self::Currency>,
		data: Self::Data,
	) -> Option<Self::Extrinsic<ContractCall<Self>>> {
		compose_extrinsic!(
			self,
			CONTRACTS_MODULE,
			CALL,
			dest,
			Compact(value),
			gas_limit,
			storage_deposit_limit.map(Compact),
			data
		)
	}

	async fn contract_instantiate_with_code(
		&self,
		value: Self::Currency,
		gas_limit: Self::Weight,
		storage_deposit_limit: Option<Self::Currency>,
		code: Self::Code,
		data: Self::Data,
		salt: Self::Salt,
	) -> Option<Self::Extrinsic<InstantiateWithCodeCall<Self>>> {
		compose_extrinsic!(
			self,
			CONTRACTS_MODULE,
			INSTANTIATE_WITH_CODE,
			Compact(value),
			gas_limit,
			storage_deposit_limit.map(Compact),
			code,
			data,
			salt
		)
	}

	async fn contract_instantiate(
		&self,
		value: Self::Currency,
		gas_limit: Self::Weight,
		storage_deposit_limit: Option<Self::Currency>,
		code_hash: Self::CodeHash,
		data: Self::Data,
		salt: Self::Salt,
	) -> Option<Self::Extrinsic<InstantiateCall<Self>>> {
		compose_extrinsic!(
			self,
			CONTRACTS_MODULE,
			INSTANTIATE,
			Compact(value),
			gas_limit,
			storage_deposit_limit.map(Compact),
			code_hash,
			data,
			salt
		)
	}

	async fn contract_migrate(
		&self,
		weight_limit: Self::Weight,
	) -> Option<Self::Extrinsic<MigrateCall<Self>>> {
		compose_extrinsic!(self, CONTRACTS_MODULE, MIGRATE, weight_limit)
	}
}
