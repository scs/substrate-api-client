pub use staking::RewardDestination;

use codec::Compact;
use sp_core::Pair;
use sp_runtime::{MultiSignature, MultiSigner};

use crate::extrinsic::balances::Balance;
use crate::extrinsic::CallIndex;
use crate::{compose_extrinsic, Api, GenericAddress, RpcClient, UncheckedExtrinsicV4};

const STAKING_MODULE: &str = "Staking";
const STAKING_BOND: &str = "bond";
const STAKING_BOND_EXTRA: &str = "bond_extra";
const STAKING_UNBOND: &str = "unbond";
const STAKING_REBOND: &str = "rebond";
const STAKING_WITHDRAW_UNBONDED: &str = "withdraw_unbonded";
const STAKING_NOMINATE: &str = "nominate";
const STAKING_CHILL: &str = "chill";

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

pub type StakingBondXt = UncheckedExtrinsicV4<StakingBondFn>;
pub type StakingBondExtraXt = UncheckedExtrinsicV4<StakingBondExtraFn>;
pub type StakingUnbondXt = UncheckedExtrinsicV4<StakingUnbondFn>;
pub type StakingRebondXt = UncheckedExtrinsicV4<StakingRebondFn>;
pub type StakingWithdrawUnbondedXt = UncheckedExtrinsicV4<StakingWithdrawUnbondedFn>;
pub type StakingNominateXt = UncheckedExtrinsicV4<StakingNominateFn>;
pub type StakingChillXt = UncheckedExtrinsicV4<StakingChillFn>;

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
}
