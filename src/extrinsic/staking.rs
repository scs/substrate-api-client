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

//! Extrinsics for `pallet-staking`.

use super::common::*;
use crate::{rpc::Request, Api};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{
	BalancesConfig, CallIndex, ExtrinsicParams, RewardDestination, SignExtrinsic, StakingConfig,
	UncheckedExtrinsicV4,
};
use codec::{Compact, Encode};
use serde::de::DeserializeOwned;
use sp_runtime::traits::GetRuntimeBlockType;

const STAKING_MODULE: &str = "Staking";
const STAKING_BOND: &str = "bond";
const STAKING_BOND_EXTRA: &str = "bond_extra";
const STAKING_UNBOND: &str = "unbond";
const STAKING_REBOND: &str = "rebond";
const STAKING_WITHDRAW_UNBONDED: &str = "withdraw_unbonded";
const STAKING_NOMINATE: &str = "nominate";
const STAKING_CHILL: &str = "chill";
const STAKING_SET_CONTROLLER: &str = "set_controller";
const PAYOUT_STAKERS: &str = "payout_stakers";
const FORCE_NEW_ERA: &str = "force_new_era";
const FORCE_NEW_ERA_ALWAYS: &str = "force_new_era_always";
const FORCE_NO_ERA: &str = "force_no_era";
const STAKING_SET_PAYEE: &str = "set_payee";
const SET_VALIDATOR_COUNT: &str = "set_validator_count";

pub type StakingBondFn<Address, Balance> =
	(CallIndex, Address, Compact<Balance>, RewardDestination<Address>);
pub type StakingBondExtraFn<Balance> = (CallIndex, Compact<Balance>);
pub type StakingUnbondFn<Balance> = (CallIndex, Compact<Balance>);
pub type StakingRebondFn<Balance> = (CallIndex, Compact<Balance>);
pub type StakingWithdrawUnbondedFn = (CallIndex, u32);
pub type StakingNominateFn<Address> = (CallIndex, Vec<Address>);
pub type StakingChillFn = CallIndex;
pub type StakingSetControllerFn<Address> = (CallIndex, Address);
pub type StakingPayoutStakersFn<AccountId> = (CallIndex, PayoutStakers<AccountId>);
pub type StakingForceNewEraFn = (CallIndex, ForceEra);
pub type StakingForceNewEraAlwaysFn = (CallIndex, ForceEra);
pub type StakingForceNoEraFn = (CallIndex, ForceEra);
pub type StakingSetPayeeFn<Address> = (CallIndex, Address);
pub type StakingSetValidatorCountFn = (CallIndex, u32);

pub type StakingBondXt<Address, Signature, SignedExtra, Balance> =
	UncheckedExtrinsicV4<Address, StakingBondFn<Address, Balance>, Signature, SignedExtra>;
pub type StakingBondExtraXt<Address, Signature, SignedExtra, Balance> =
	UncheckedExtrinsicV4<Address, StakingBondExtraFn<Balance>, Signature, SignedExtra>;
pub type StakingUnbondXt<Address, Signature, SignedExtra, Balance> =
	UncheckedExtrinsicV4<Address, StakingUnbondFn<Balance>, Signature, SignedExtra>;
pub type StakingRebondXt<Address, Signature, SignedExtra, Balance> =
	UncheckedExtrinsicV4<Address, StakingRebondFn<Balance>, Signature, SignedExtra>;
pub type StakingWithdrawUnbondedXt<Address, Signature, SignedExtra> =
	UncheckedExtrinsicV4<Address, StakingWithdrawUnbondedFn, Signature, SignedExtra>;
pub type StakingNominateXt<Address, Signature, SignedExtra> =
	UncheckedExtrinsicV4<Address, StakingNominateFn<Address>, Signature, SignedExtra>;
pub type StakingChillXt<Address, Signature, SignedExtra> =
	UncheckedExtrinsicV4<Address, StakingChillFn, Signature, SignedExtra>;
pub type StakingSetControllerXt<Address, Signature, SignedExtra> =
	UncheckedExtrinsicV4<Address, StakingSetControllerFn<Address>, Signature, SignedExtra>;
pub type StakingPayoutStakersXt<Address, Signature, SignedExtra, AccountId> =
	UncheckedExtrinsicV4<Address, StakingPayoutStakersFn<AccountId>, Signature, SignedExtra>;
pub type StakingForceNewEraXt<Address, Signature, SignedExtra> =
	UncheckedExtrinsicV4<Address, StakingForceNewEraFn, Signature, SignedExtra>;
pub type StakingForceNewEraAlwaysXt<Address, Signature, SignedExtra> =
	UncheckedExtrinsicV4<Address, StakingForceNewEraAlwaysFn, Signature, SignedExtra>;
pub type StakingForceNoEraXt<Address, Signature, SignedExtra> =
	UncheckedExtrinsicV4<Address, StakingForceNoEraFn, Signature, SignedExtra>;
pub type StakingSetPayeeXt<Address, Signature, SignedExtra> =
	UncheckedExtrinsicV4<Address, StakingSetPayeeFn<Address>, Signature, SignedExtra>;
pub type StakingSetValidatorCountXt<Address, Signature, SignedExtra> =
	UncheckedExtrinsicV4<Address, StakingSetValidatorCountFn, Signature, SignedExtra>;

// https://polkadot.js.org/docs/substrate/extrinsics#staking
impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Signer: SignExtrinsic<Runtime::AccountId>,
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: GetRuntimeBlockType + BalancesConfig + StakingConfig,
	Compact<Runtime::CurrencyBalance>: Encode,
	Runtime::Header: DeserializeOwned,
	Runtime::RuntimeBlock: DeserializeOwned,
{
	/// Bond `value` amount to `controller`
	pub fn staking_bond(
		&self,
		controller: Signer::ExtrinsicAddress,
		value: Runtime::CurrencyBalance,
		payee: RewardDestination<Signer::ExtrinsicAddress>,
	) -> StakingBondXt<
		Signer::ExtrinsicAddress,
		Signer::Signature,
		Params::SignedExtra,
		Runtime::CurrencyBalance,
	> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_BOND, controller, Compact(value), payee)
	}

	/// Bonds extra funds from the stash's free balance to the balance for staking.
	pub fn staking_bond_extra(
		&self,
		value: Runtime::CurrencyBalance,
	) -> StakingBondExtraXt<
		Signer::ExtrinsicAddress,
		Signer::Signature,
		Params::SignedExtra,
		Runtime::CurrencyBalance,
	> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_BOND_EXTRA, Compact(value))
	}

	/// Unbond `value` portion of the stash.
	/// If `value` is less than the minimum required, then the entire amount is unbound.
	/// Must be signed by the controller of the stash.
	pub fn staking_unbond(
		&self,
		value: Runtime::CurrencyBalance,
	) -> StakingUnbondXt<
		Signer::ExtrinsicAddress,
		Signer::Signature,
		Params::SignedExtra,
		Runtime::CurrencyBalance,
	> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_UNBOND, Compact(value))
	}

	/// Rebond `value` portion of the current amount that is in the process of unbonding.
	pub fn staking_rebond(
		&self,
		value: Runtime::CurrencyBalance,
	) -> StakingRebondXt<
		Signer::ExtrinsicAddress,
		Signer::Signature,
		Params::SignedExtra,
		Runtime::CurrencyBalance,
	> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_REBOND, Compact(value))
	}

	/// Free the balance of the stash so the stash account can do whatever it wants.
	/// Must be signed by the controller of the stash and called when EraElectionStatus is Closed.
	/// For most users, `num_slashing_spans` should be 0.
	pub fn staking_withdraw_unbonded(
		&self,
		num_slashing_spans: u32,
	) -> StakingWithdrawUnbondedXt<Signer::ExtrinsicAddress, Signer::Signature, Params::SignedExtra>
	{
		compose_extrinsic!(self, STAKING_MODULE, STAKING_WITHDRAW_UNBONDED, num_slashing_spans)
	}

	/// Nominate `targets` as validators.
	/// Must be signed by the controller of the stash and called when EraElectionStatus is Closed.
	pub fn staking_nominate(
		&self,
		targets: Vec<Signer::ExtrinsicAddress>,
	) -> StakingNominateXt<Signer::ExtrinsicAddress, Signer::Signature, Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_NOMINATE, targets)
	}

	/// Stop nominating por validating. Effects take place in the next era
	pub fn staking_chill(
		&self,
	) -> StakingChillXt<Signer::ExtrinsicAddress, Signer::Signature, Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_CHILL)
	}

	/// (Re-)set the controller of the stash
	/// Effects will be felt at the beginning of the next era.
	/// Must be Signed by the stash, not the controller.
	pub fn staking_set_controller(
		&self,
		controller: Signer::ExtrinsicAddress,
	) -> StakingSetControllerXt<Signer::ExtrinsicAddress, Signer::Signature, Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_SET_CONTROLLER, controller)
	}

	/// Return the payout call for the given era
	pub fn payout_stakers(
		&self,
		era: u32,
		account: Runtime::AccountId,
	) -> StakingPayoutStakersXt<
		Signer::ExtrinsicAddress,
		Signer::Signature,
		Params::SignedExtra,
		Runtime::AccountId,
	> {
		let value = PayoutStakers { validator_stash: account, era };
		compose_extrinsic!(self, STAKING_MODULE, PAYOUT_STAKERS, value)
	}

	/// For New Era at the end of Next Session.
	pub fn force_new_era(
		&self,
	) -> StakingForceNewEraXt<Signer::ExtrinsicAddress, Signer::Signature, Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, FORCE_NEW_ERA, ForceEra {})
	}

	/// Force there to be a new era at the end of sessions indefinitely.
	pub fn force_new_era_always(
		&self,
	) -> StakingForceNewEraAlwaysXt<Signer::ExtrinsicAddress, Signer::Signature, Params::SignedExtra>
	{
		compose_extrinsic!(self, STAKING_MODULE, FORCE_NEW_ERA_ALWAYS, ForceEra {})
	}

	/// Force there to be no new eras indefinitely.
	pub fn force_no_era(
		&self,
	) -> StakingForceNewEraAlwaysXt<Signer::ExtrinsicAddress, Signer::Signature, Params::SignedExtra>
	{
		compose_extrinsic!(self, STAKING_MODULE, FORCE_NO_ERA, ForceEra {})
	}

	/// Re-set the payment target for a controller.
	pub fn set_payee(
		&self,
		payee: Signer::ExtrinsicAddress,
	) -> StakingSetControllerXt<Signer::ExtrinsicAddress, Signer::Signature, Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_SET_PAYEE, payee)
	}

	/// Sets the number of validators.
	pub fn set_validator_count(
		&self,
		count: u32,
	) -> StakingSetValidatorCountXt<Signer::ExtrinsicAddress, Signer::Signature, Params::SignedExtra>
	{
		compose_extrinsic!(self, STAKING_MODULE, SET_VALIDATOR_COUNT, count)
	}
}
