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

use crate::{Api, RpcClient};
use ac_compose_macros::compose_extrinsic;
use ac_primitives::{Balance, CallIndex, GenericAddress, UncheckedExtrinsicV4};
use codec::Compact;
use sp_core::Pair;
use sp_runtime::{MultiSignature, MultiSigner};

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

pub type StakingBondFn = (
    CallIndex,
    GenericAddress,
    Compact<Balance>,
    RewardDestination<GenericAddress>,
);
pub type StakingBondExtraFn = (CallIndex, Compact<Balance>);
pub type StakingUnbondFn = (CallIndex, Compact<Balance>);
pub type StakingRebondFn = (CallIndex, Compact<Balance>);
pub type StakingWithdrawUnbondedFn = (CallIndex, u32);
pub type StakingNominateFn = (CallIndex, Vec<GenericAddress>);
pub type StakingChillFn = CallIndex;
pub type StakingSetControllerFn = (CallIndex, GenericAddress);

pub type StakingBondXt = UncheckedExtrinsicV4<StakingBondFn>;
pub type StakingBondExtraXt = UncheckedExtrinsicV4<StakingBondExtraFn>;
pub type StakingUnbondXt = UncheckedExtrinsicV4<StakingUnbondFn>;
pub type StakingRebondXt = UncheckedExtrinsicV4<StakingRebondFn>;
pub type StakingWithdrawUnbondedXt = UncheckedExtrinsicV4<StakingWithdrawUnbondedFn>;
pub type StakingNominateXt = UncheckedExtrinsicV4<StakingNominateFn>;
pub type StakingChillXt = UncheckedExtrinsicV4<StakingChillFn>;
pub type StakingSetControllerXt = UncheckedExtrinsicV4<StakingSetControllerFn>;

// https://polkadot.js.org/docs/substrate/extrinsics#staking
impl<P, Client> Api<P, Client>
where
    P: Pair,
    MultiSignature: From<P::Signature>,
    MultiSigner: From<P::Public>,
    Client: RpcClient,
{
    /// Bond `value` amount to `controller`
    pub fn staking_bond(
        &self,
        controller: GenericAddress,
        value: Balance,
        payee: RewardDestination<GenericAddress>,
    ) -> StakingBondXt {
        compose_extrinsic!(
            self,
            STAKING_MODULE,
            STAKING_BOND,
            controller,
            Compact(value),
            payee
        )
    }

    /// Bonds extra funds from the stash's free balance to the balance for staking.
    pub fn staking_bond_extra(&self, value: Balance) -> StakingBondExtraXt {
        compose_extrinsic!(self, STAKING_MODULE, STAKING_BOND_EXTRA, Compact(value))
    }

    /// Unbond `value` portion of the stash.
    /// If `value` is less than the minimum required, then the entire amount is unbound.
    /// Must be signed by the controller of the stash.
    pub fn staking_unbond(&self, value: Balance) -> StakingUnbondXt {
        compose_extrinsic!(self, STAKING_MODULE, STAKING_UNBOND, Compact(value))
    }

    /// Rebond `value` portion of the current amount that is in the process of unbonding.
    pub fn staking_rebond(&self, value: Balance) -> StakingRebondXt {
        compose_extrinsic!(self, STAKING_MODULE, STAKING_REBOND, Compact(value))
    }

    /// Free the balance of the stash so the stash account can do whatever it wants.
    /// Must be signed by the controller of the stash and called when EraElectionStatus is Closed.
    /// For most users, `num_slashing_spans` should be 0.
    pub fn staking_withdraw_unbonded(&self, num_slashing_spans: u32) -> StakingWithdrawUnbondedXt {
        compose_extrinsic!(
            self,
            STAKING_MODULE,
            STAKING_WITHDRAW_UNBONDED,
            num_slashing_spans
        )
    }

    /// Nominate `targets` as validators.
    /// Must be signed by the controller of the stash and called when EraElectionStatus is Closed.
    pub fn staking_nominate(&self, targets: Vec<GenericAddress>) -> StakingNominateXt {
        compose_extrinsic!(self, STAKING_MODULE, STAKING_NOMINATE, targets)
    }

    /// Stop nominating por validating. Effects take place in the next era
    pub fn staking_chill(&self) -> StakingChillXt {
        compose_extrinsic!(self, STAKING_MODULE, STAKING_CHILL)
    }

    /// (Re-)set the controller of the stash
    /// Effects will be felt at the beginning of the next era.
    /// Must be Signed by the stash, not the controller.
    pub fn staking_set_controller(&self, controller: GenericAddress) -> StakingSetControllerXt {
        compose_extrinsic!(self, STAKING_MODULE, STAKING_SET_CONTROLLER, controller)
    }
}
