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

use crate::{rpc::Request, Api};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{
	CallIndex, ExtrinsicParams, RewardDestination, SignExtrinsic, StakingConfig,
	UncheckedExtrinsicV4,
};
use codec::{Compact, Decode, Encode};

const STAKING_MODULE: &str = "Staking";
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

#[maybe_async::maybe_async(?Send)]
pub trait StakingExtrinsics {
	type Balance;
	type RewardDestination;
	type AccountId;
	type Address;
	type Extrinsic<Call>;

	/// Bond `value` amount to `controller`.
	async fn staking_bond(
		&self,
		controller: Self::Address,
		value: Self::Balance,
		payee: Self::RewardDestination,
	) -> Self::Extrinsic<BondCall<Self::Address, Self::Balance>>;

	/// Bonds extra funds from the stash's free balance to the balance for staking.
	async fn staking_bond_extra(
		&self,
		value: Self::Balance,
	) -> Self::Extrinsic<BondExtraCall<Self::Balance>>;

	/// Unbond `value` portion of the stash.
	/// If `value` is less than the minimum required, then the entire amount is unbound.
	/// Must be signed by the controller of the stash.
	async fn staking_unbond(
		&self,
		value: Self::Balance,
	) -> Self::Extrinsic<UnbondCall<Self::Balance>>;

	/// Rebond `value` portion of the current amount that is in the process of unbonding.
	async fn staking_rebond(
		&self,
		value: Self::Balance,
	) -> Self::Extrinsic<RebondCall<Self::Balance>>;

	/// Free the balance of the stash so the stash account can do whatever it wants.
	/// Must be signed by the controller of the stash and called when EraElectionStatus is Closed.
	/// For most users, `num_slashing_spans` should be 0.
	async fn staking_withdraw_unbonded(
		&self,
		num_slashing_spans: u32,
	) -> Self::Extrinsic<WithdrawUnbondedCall>;

	/// Nominate `targets` as validators.
	/// Must be signed by the controller of the stash and called when EraElectionStatus is Closed.
	async fn staking_nominate(
		&self,
		targets: Vec<Self::Address>,
	) -> Self::Extrinsic<NominateCall<Self::Address>>;

	/// Stop nominating por validating. Effects take place in the next era
	async fn staking_chill(&self) -> Self::Extrinsic<ChillCall>;

	/// (Re-)set the controller of the stash
	/// Effects will be felt at the beginning of the next era.
	/// Must be Signed by the stash, not the controller.
	async fn staking_set_controller(
		&self,
		controller: Self::Address,
	) -> Self::Extrinsic<SetControllerCall<Self::Address>>;

	/// Return the payout call for the given era
	async fn payout_stakers(
		&self,
		era: u32,
		account: Self::AccountId,
	) -> Self::Extrinsic<PayoutStakersCall<Self::AccountId>>;

	/// For New Era at the end of Next Session.
	async fn force_new_era(&self) -> Self::Extrinsic<ForceNewEraCall>;

	/// Force there to be a new era at the end of sessions indefinitely.
	async fn force_new_era_always(&self) -> Self::Extrinsic<ForceNewEraAlwaysCall>;

	/// Force there to be no new eras indefinitely.
	async fn force_no_era(&self) -> Self::Extrinsic<ForceNewEraAlwaysCall>;

	/// Re-set the payment target for a controller.
	async fn set_payee(&self, payee: Self::Address)
		-> Self::Extrinsic<SetPayeeCall<Self::Address>>;

	/// Sets the number of validators.
	async fn set_validator_count(&self, count: u32) -> Self::Extrinsic<SetValidatorCountCall>;
}

#[maybe_async::maybe_async(?Send)]
impl<Signer, Client, Params, Runtime> StakingExtrinsics for Api<Signer, Client, Params, Runtime>
where
	Signer: SignExtrinsic<Runtime::AccountId>,
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: StakingConfig,
	Compact<Runtime::CurrencyBalance>: Encode,
{
	type Balance = Runtime::CurrencyBalance;
	type RewardDestination = RewardDestination<Self::Address>;
	type AccountId = Runtime::AccountId;
	type Address = Signer::ExtrinsicAddress;
	type Extrinsic<Call> =
		UncheckedExtrinsicV4<Self::Address, Call, Signer::Signature, Params::SignedExtra>;

	async fn staking_bond(
		&self,
		controller: Self::Address,
		value: Self::Balance,
		payee: Self::RewardDestination,
	) -> Self::Extrinsic<BondCall<Self::Address, Self::Balance>> {
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, BOND, controller, Compact(value), payee)
	}

	async fn staking_bond_extra(
		&self,
		value: Self::Balance,
	) -> Self::Extrinsic<BondExtraCall<Self::Balance>> {
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, BOND_EXTRA, Compact(value))
	}

	async fn staking_unbond(
		&self,
		value: Self::Balance,
	) -> Self::Extrinsic<UnbondCall<Self::Balance>> {
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, UNBOND, Compact(value))
	}

	async fn staking_rebond(
		&self,
		value: Self::Balance,
	) -> Self::Extrinsic<RebondCall<Self::Balance>> {
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, REBOND, Compact(value))
	}

	async fn staking_withdraw_unbonded(
		&self,
		num_slashing_spans: u32,
	) -> Self::Extrinsic<WithdrawUnbondedCall> {
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, WITHDRAW_UNBONDED, num_slashing_spans)
	}

	async fn staking_nominate(
		&self,
		targets: Vec<Self::Address>,
	) -> Self::Extrinsic<NominateCall<Self::Address>> {
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, NOMINATE, targets)
	}

	async fn staking_chill(&self) -> Self::Extrinsic<ChillCall> {
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, CHILL)
	}

	async fn staking_set_controller(
		&self,
		controller: Self::Address,
	) -> Self::Extrinsic<SetControllerCall<Self::Address>> {
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, SET_CONTROLLER, controller)
	}

	async fn payout_stakers(
		&self,
		era: u32,
		account: Self::AccountId,
	) -> Self::Extrinsic<PayoutStakersCall<Self::AccountId>> {
		let value = PayoutStakers { validator_stash: account, era };
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, PAYOUT_STAKERS, value)
	}

	async fn force_new_era(&self) -> Self::Extrinsic<ForceNewEraCall> {
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, FORCE_NEW_ERA)
	}

	async fn force_new_era_always(&self) -> Self::Extrinsic<ForceNewEraAlwaysCall> {
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, FORCE_NEW_ERA_ALWAYS)
	}

	async fn force_no_era(&self) -> Self::Extrinsic<ForceNewEraAlwaysCall> {
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, FORCE_NO_ERA)
	}

	async fn set_payee(
		&self,
		payee: Self::Address,
	) -> Self::Extrinsic<SetPayeeCall<Self::Address>> {
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, SET_PAYEE, payee)
	}

	async fn set_validator_count(&self, count: u32) -> Self::Extrinsic<SetValidatorCountCall> {
		let nonce = self.get_nonce().await.unwrap();
		compose_extrinsic!(self, nonce, STAKING_MODULE, SET_VALIDATOR_COUNT, count)
	}
}
