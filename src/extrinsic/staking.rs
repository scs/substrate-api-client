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
//! https://polkadot.js.org/docs/substrate/extrinsics#staking

use super::{AssignExtrinsicTypes, ExtrinsicFor};
use crate::{rpc::Request, Api};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{
	BalancesConfig, CallIndex, ExtrinsicParams, RewardDestination, SignExtrinsic, StakingConfig,
};
use codec::{Compact, Decode, Encode};
use serde::de::DeserializeOwned;
use sp_runtime::traits::GetRuntimeBlockType;

const MODULE: &str = "Staking";
const BOND: &str = "bond";
const BOND_EXTRA: &str = "bond_extra";
const UNBOND: &str = "unbond";
const REBOND: &str = "rebond";
const WITHDRAW_UNBONDED: &str = "withdraw_unbonded";
const NOMINATE: &str = "nominate";
const CHILL: &str = "chill";
const SET_CONTROLLER: &str = "set_controller";
const PAYOUT_STAKERS: &str = "payout_stakers";
const FORCE_NEW_ERA: &str = "force_new_era";
const FORCE_NEW_ERA_ALWAYS: &str = "force_new_era_always";
const FORCE_NO_ERA: &str = "force_no_era";
const SET_PAYEE: &str = "set_payee";
const SET_VALIDATOR_COUNT: &str = "set_validator_count";

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug)]
pub struct PayoutStakers<AccountId> {
	pub validator_stash: AccountId,
	pub era: u32,
}

pub type BondCall<Address, Balance> =
	(CallIndex, Address, Compact<Balance>, RewardDestination<Address>);
pub type BondExtraCall<Balance> = (CallIndex, Compact<Balance>);
pub type UnbondCall<Balance> = (CallIndex, Compact<Balance>);
pub type RebondCall<Balance> = (CallIndex, Compact<Balance>);
pub type WithdrawUnbondedCall = (CallIndex, u32);
pub type NominateCall<Address> = (CallIndex, Vec<Address>);
pub type ChillCall = CallIndex;
pub type SetControllerCall<Address> = (CallIndex, Address);
pub type PayoutStakersCall<AccountId> = (CallIndex, PayoutStakers<AccountId>);
pub type ForceNewEraCall = CallIndex;
pub type ForceNewEraAlwaysCall = CallIndex;
pub type ForceNoEraCall = CallIndex;
pub type SetPayeeCall<Address> = (CallIndex, Address);
pub type SetValidatorCountCall = (CallIndex, u32);

pub trait CreateStakingExtrinsic: AssignExtrinsicTypes {
	type Balance;
	type RewardDestination;
	type AccountId;

	/// Bond `value` amount to `controller`.
	fn staking_bond(
		&self,
		controller: Self::Address,
		value: Self::Balance,
		payee: Self::RewardDestination,
	) -> ExtrinsicFor<Self, BondCall<Self::Address, Self::Balance>>;

	/// Bonds extra funds from the stash's free balance to the balance for staking.
	fn staking_bond_extra(
		&self,
		value: Self::Balance,
	) -> ExtrinsicFor<Self, BondExtraCall<Self::Balance>>;

	/// Unbond `value` portion of the stash.
	/// If `value` is less than the minimum required, then the entire amount is unbound.
	/// Must be signed by the controller of the stash.
	fn staking_unbond(&self, value: Self::Balance)
		-> ExtrinsicFor<Self, UnbondCall<Self::Balance>>;

	/// Rebond `value` portion of the current amount that is in the process of unbonding.
	fn staking_rebond(&self, value: Self::Balance)
		-> ExtrinsicFor<Self, RebondCall<Self::Balance>>;

	/// Free the balance of the stash so the stash account can do whatever it wants.
	/// Must be signed by the controller of the stash and called when EraElectionStatus is Closed.
	/// For most users, `num_slashing_spans` should be 0.
	fn staking_withdraw_unbonded(
		&self,
		num_slashing_spans: u32,
	) -> ExtrinsicFor<Self, WithdrawUnbondedCall>;

	/// Nominate `targets` as validators.
	/// Must be signed by the controller of the stash and called when EraElectionStatus is Closed.
	fn staking_nominate(
		&self,
		targets: Vec<Self::Address>,
	) -> ExtrinsicFor<Self, NominateCall<Self::Address>>;

	/// Stop nominating por validating. Effects take place in the next era
	fn staking_chill(&self) -> ExtrinsicFor<Self, ChillCall>;

	/// (Re-)set the controller of the stash
	/// Effects will be felt at the beginning of the next era.
	/// Must be Signed by the stash, not the controller.
	fn staking_set_controller(
		&self,
		controller: Self::Address,
	) -> ExtrinsicFor<Self, SetControllerCall<Self::Address>>;

	/// Return the payout call for the given era
	fn payout_stakers(
		&self,
		era: u32,
		account: Self::AccountId,
	) -> ExtrinsicFor<Self, PayoutStakersCall<Self::AccountId>>;

	/// For New Era at the end of Next Session.
	fn force_new_era(&self) -> ExtrinsicFor<Self, ForceNewEraCall>;

	/// Force there to be a new era at the end of sessions indefinitely.
	fn force_new_era_always(&self) -> ExtrinsicFor<Self, ForceNewEraAlwaysCall>;

	/// Force there to be no new eras indefinitely.
	fn force_no_era(&self) -> ExtrinsicFor<Self, ForceNewEraAlwaysCall>;

	/// Re-set the payment target for a controller.
	fn set_payee(&self, payee: Self::Address) -> ExtrinsicFor<Self, SetPayeeCall<Self::Address>>;

	/// Sets the number of validators.
	fn set_validator_count(&self, count: u32) -> ExtrinsicFor<Self, SetValidatorCountCall>;
}

impl<Signer, Client, Params, Runtime> CreateStakingExtrinsic
	for Api<Signer, Client, Params, Runtime>
where
	Signer: SignExtrinsic<Runtime::AccountId>,
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: GetRuntimeBlockType + BalancesConfig + StakingConfig,
	Compact<Runtime::CurrencyBalance>: Encode,
	Runtime::Header: DeserializeOwned,
	Runtime::RuntimeBlock: DeserializeOwned,
{
	type Balance = Runtime::CurrencyBalance;
	type RewardDestination = RewardDestination<Self::Address>;
	type AccountId = Runtime::AccountId;

	fn staking_bond(
		&self,
		controller: Self::Address,
		value: Self::Balance,
		payee: Self::RewardDestination,
	) -> ExtrinsicFor<Self, BondCall<Self::Address, Self::Balance>> {
		compose_extrinsic!(self, MODULE, BOND, controller, Compact(value), payee)
	}

	fn staking_bond_extra(
		&self,
		value: Self::Balance,
	) -> ExtrinsicFor<Self, BondExtraCall<Self::Balance>> {
		compose_extrinsic!(self, MODULE, BOND_EXTRA, Compact(value))
	}

	fn staking_unbond(
		&self,
		value: Self::Balance,
	) -> ExtrinsicFor<Self, UnbondCall<Self::Balance>> {
		compose_extrinsic!(self, MODULE, UNBOND, Compact(value))
	}

	fn staking_rebond(
		&self,
		value: Self::Balance,
	) -> ExtrinsicFor<Self, RebondCall<Self::Balance>> {
		compose_extrinsic!(self, MODULE, REBOND, Compact(value))
	}

	fn staking_withdraw_unbonded(
		&self,
		num_slashing_spans: u32,
	) -> ExtrinsicFor<Self, WithdrawUnbondedCall> {
		compose_extrinsic!(self, MODULE, WITHDRAW_UNBONDED, num_slashing_spans)
	}

	fn staking_nominate(
		&self,
		targets: Vec<Self::Address>,
	) -> ExtrinsicFor<Self, NominateCall<Self::Address>> {
		compose_extrinsic!(self, MODULE, NOMINATE, targets)
	}

	fn staking_chill(&self) -> ExtrinsicFor<Self, ChillCall> {
		compose_extrinsic!(self, MODULE, CHILL)
	}

	fn staking_set_controller(
		&self,
		controller: Self::Address,
	) -> ExtrinsicFor<Self, SetControllerCall<Self::Address>> {
		compose_extrinsic!(self, MODULE, SET_CONTROLLER, controller)
	}

	fn payout_stakers(
		&self,
		era: u32,
		account: Self::AccountId,
	) -> ExtrinsicFor<Self, PayoutStakersCall<Self::AccountId>> {
		let value = PayoutStakers { validator_stash: account, era };
		compose_extrinsic!(self, MODULE, PAYOUT_STAKERS, value)
	}

	fn force_new_era(&self) -> ExtrinsicFor<Self, ForceNewEraCall> {
		compose_extrinsic!(self, MODULE, FORCE_NEW_ERA)
	}

	fn force_new_era_always(&self) -> ExtrinsicFor<Self, ForceNewEraAlwaysCall> {
		compose_extrinsic!(self, MODULE, FORCE_NEW_ERA_ALWAYS)
	}

	fn force_no_era(&self) -> ExtrinsicFor<Self, ForceNewEraAlwaysCall> {
		compose_extrinsic!(self, MODULE, FORCE_NO_ERA)
	}

	fn set_payee(&self, payee: Self::Address) -> ExtrinsicFor<Self, SetPayeeCall<Self::Address>> {
		compose_extrinsic!(self, MODULE, SET_PAYEE, payee)
	}

	fn set_validator_count(&self, count: u32) -> ExtrinsicFor<Self, SetValidatorCountCall> {
		compose_extrinsic!(self, MODULE, SET_VALIDATOR_COUNT, count)
	}
}
