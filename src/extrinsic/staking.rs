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
use crate::{Api, RpcClient};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{Balance, CallIndex, ExtrinsicParams, GenericAddress, UncheckedExtrinsicV4};
use codec::Compact;
use sp_core::Pair;
use sp_runtime::{AccountId32, MultiSignature, MultiSigner};

pub use staking::RewardDestination;

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

pub type StakingBondFn =
	(CallIndex, GenericAddress, Compact<Balance>, RewardDestination<GenericAddress>);
pub type StakingBondExtraFn = (CallIndex, Compact<Balance>);
pub type StakingUnbondFn = (CallIndex, Compact<Balance>);
pub type StakingRebondFn = (CallIndex, Compact<Balance>);
pub type StakingWithdrawUnbondedFn = (CallIndex, u32);
pub type StakingNominateFn = (CallIndex, Vec<GenericAddress>);
pub type StakingChillFn = CallIndex;
pub type StakingSetControllerFn = (CallIndex, GenericAddress);
pub type StakingPayoutStakersFn = (CallIndex, PayoutStakers);
pub type StakingForceNewEraFn = (CallIndex, ForceEra);
pub type StakingForceNewEraAlwaysFn = (CallIndex, ForceEra);
pub type StakingForceNoEraFn = (CallIndex, ForceEra);
pub type StakingSetPayeeFn = (CallIndex, GenericAddress);
pub type StakingSetValidatorCountFn = (CallIndex, u32);

pub type StakingBondXt<SignedExtra> = UncheckedExtrinsicV4<StakingBondFn, SignedExtra>;
pub type StakingBondExtraXt<SignedExtra> = UncheckedExtrinsicV4<StakingBondExtraFn, SignedExtra>;
pub type StakingUnbondXt<SignedExtra> = UncheckedExtrinsicV4<StakingUnbondFn, SignedExtra>;
pub type StakingRebondXt<SignedExtra> = UncheckedExtrinsicV4<StakingRebondFn, SignedExtra>;
pub type StakingWithdrawUnbondedXt<SignedExtra> =
	UncheckedExtrinsicV4<StakingWithdrawUnbondedFn, SignedExtra>;
pub type StakingNominateXt<SignedExtra> = UncheckedExtrinsicV4<StakingNominateFn, SignedExtra>;
pub type StakingChillXt<SignedExtra> = UncheckedExtrinsicV4<StakingChillFn, SignedExtra>;
pub type StakingSetControllerXt<SignedExtra> =
	UncheckedExtrinsicV4<StakingSetControllerFn, SignedExtra>;
pub type StakingPayoutStakersXt<SignedExtra> =
	UncheckedExtrinsicV4<StakingPayoutStakersFn, SignedExtra>;
pub type StakingForceNewEraXt<SignedExtra> =
	UncheckedExtrinsicV4<StakingForceNewEraFn, SignedExtra>;
pub type StakingForceNewEraAlwaysXt<SignedExtra> =
	UncheckedExtrinsicV4<StakingForceNewEraAlwaysFn, SignedExtra>;
pub type StakingForceNoEraXt<SignedExtra> = UncheckedExtrinsicV4<StakingForceNoEraFn, SignedExtra>;
pub type StakingSetPayeeXt<SignedExtra> = UncheckedExtrinsicV4<StakingSetPayeeFn, SignedExtra>;
pub type StakingSetValidatorCountXt<SignedExtra> =
	UncheckedExtrinsicV4<StakingSetValidatorCountFn, SignedExtra>;

// https://polkadot.js.org/docs/substrate/extrinsics#staking
impl<P, Client, Params> Api<P, Client, Params>
where
	P: Pair,
	MultiSignature: From<P::Signature>,
	MultiSigner: From<P::Public>,
	Client: RpcClient,
	Params: ExtrinsicParams,
{
	/// Bond `value` amount to `controller`
	pub fn staking_bond(
		&self,
		controller: GenericAddress,
		value: Balance,
		payee: RewardDestination<GenericAddress>,
	) -> StakingBondXt<Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_BOND, controller, Compact(value), payee)
	}

	/// Bonds extra funds from the stash's free balance to the balance for staking.
	pub fn staking_bond_extra(&self, value: Balance) -> StakingBondExtraXt<Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_BOND_EXTRA, Compact(value))
	}

	/// Unbond `value` portion of the stash.
	/// If `value` is less than the minimum required, then the entire amount is unbound.
	/// Must be signed by the controller of the stash.
	pub fn staking_unbond(&self, value: Balance) -> StakingUnbondXt<Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_UNBOND, Compact(value))
	}

	/// Rebond `value` portion of the current amount that is in the process of unbonding.
	pub fn staking_rebond(&self, value: Balance) -> StakingRebondXt<Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_REBOND, Compact(value))
	}

	/// Free the balance of the stash so the stash account can do whatever it wants.
	/// Must be signed by the controller of the stash and called when EraElectionStatus is Closed.
	/// For most users, `num_slashing_spans` should be 0.
	pub fn staking_withdraw_unbonded(
		&self,
		num_slashing_spans: u32,
	) -> StakingWithdrawUnbondedXt<Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_WITHDRAW_UNBONDED, num_slashing_spans)
	}

	/// Nominate `targets` as validators.
	/// Must be signed by the controller of the stash and called when EraElectionStatus is Closed.
	pub fn staking_nominate(
		&self,
		targets: Vec<GenericAddress>,
	) -> StakingNominateXt<Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_NOMINATE, targets)
	}

	/// Stop nominating por validating. Effects take place in the next era
	pub fn staking_chill(&self) -> StakingChillXt<Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_CHILL)
	}

	/// (Re-)set the controller of the stash
	/// Effects will be felt at the beginning of the next era.
	/// Must be Signed by the stash, not the controller.
	pub fn staking_set_controller(
		&self,
		controller: GenericAddress,
	) -> StakingSetControllerXt<Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_SET_CONTROLLER, controller)
	}
	/// Return the payout call for the given era
	pub fn payout_stakers(
		&self,
		era: u32,
		account: AccountId32,
	) -> StakingPayoutStakersXt<Params::SignedExtra> {
		let value = PayoutStakers { validator_stash: account, era };
		compose_extrinsic!(self, STAKING_MODULE, PAYOUT_STAKERS, value)
	}

	/// For New Era at the end of Next Session.
	pub fn force_new_era(&self) -> StakingForceNewEraXt<Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, FORCE_NEW_ERA, ForceEra {})
	}

	/// Force there to be a new era at the end of sessions indefinitely.
	pub fn force_new_era_always(&self) -> StakingForceNewEraAlwaysXt<Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, FORCE_NEW_ERA_ALWAYS, ForceEra {})
	}

	/// Force there to be no new eras indefinitely.
	pub fn force_no_era(&self) -> StakingForceNewEraAlwaysXt<Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, FORCE_NO_ERA, ForceEra {})
	}

	/// Re-set the payment target for a controller.
	pub fn set_payee(&self, payee: GenericAddress) -> StakingSetControllerXt<Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, STAKING_SET_PAYEE, payee)
	}

	/// Sets the number of validators.
	pub fn set_validator_count(
		&self,
		count: u32,
	) -> StakingSetValidatorCountXt<Params::SignedExtra> {
		compose_extrinsic!(self, STAKING_MODULE, SET_VALIDATOR_COUNT, count)
	}
}
