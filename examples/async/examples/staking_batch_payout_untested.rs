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

use codec::{Decode, Encode};
use pallet_staking::{ActiveEraInfo, Exposure};
use sp_keyring::AccountKeyring;
use sp_runtime::{app_crypto::Ss58Codec, AccountId32};
use substrate_api_client::{
	ac_primitives::RococoRuntimeConfig,
	extrinsic::{StakingExtrinsics, UtilityExtrinsics},
	rpc::JsonrpseeClient,
	Api, GetStorage, SubmitAndWatch, XtStatus,
};

const MAX_BATCHED_TRANSACTION: u32 = 9;

// This example is currently not tested because the polkadot chain (rococo runtime) we run our example against
// does not include the staking pallet. But it still provides a good example for possible stake payouts.

pub type EraIndex = u32;

pub struct GracePeriod {
	pub enabled: bool,
	pub eras: u32,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug)]
pub struct StakingLedger {
	pub stash: AccountId32,
	#[codec(compact)]
	pub total: u128,
	#[codec(compact)]
	pub active: u128,
	pub unlocking: Vec<u32>,
	pub claimed_rewards: Vec<u32>,
}

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let alice = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<RococoRuntimeConfig, _>::new(client).await.unwrap();
	api.set_signer(alice.into());

	// Give a valid validator account address. In the kitchinsink runtime, this is Alice.
	let validator_account = AccountKeyring::Alice.to_account_id();
	// Alice Stash:
	let validator_stash =
		AccountId32::from_ss58check("5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY").unwrap();

	let active_era: ActiveEraInfo =
		api.get_storage("Staking", "ActiveEra", None).await.unwrap().unwrap();
	println!("{:?}", active_era);
	let current_era_index = active_era.index;

	// Test if payout staker extrinsic works. Careful: If tested with CI, this extrinsic will fail to be executed, because
	// one can not payout the current era (= 0 on the freshly started node). But this is okay, because we know if this
	// error is returned from the node, the extrinsic has been created correctly.
	// Sidenote: We could theoretically force a new era with sudo, but this takes at least 10 minutes ( = 1 epoch) in the
	// kitchensink rutime. We don't want to wait that long.
	let payout_staker_xt = api.payout_stakers(0, validator_stash).await.unwrap();
	let _result = api.submit_and_watch_extrinsic_until(payout_staker_xt, XtStatus::InBlock).await;

	if let Some(mut last_reward_received_at_era) =
		get_last_reward_received_for(&validator_account, current_era_index, &api).await
	{
		let grace_period = GracePeriod { enabled: false, eras: 0 };
		let mut num_of_unclaimed_payouts: u32 =
			if current_era_index - last_reward_received_at_era > 0 {
				current_era_index - last_reward_received_at_era - 1
			} else {
				0
			};
		let mut num_of_claimed_payouts = 0;
		let mut results = Vec::new();
		while num_of_unclaimed_payouts > 0 {
			let tx_limit_in_current_batch = if num_of_unclaimed_payouts > MAX_BATCHED_TRANSACTION {
				MAX_BATCHED_TRANSACTION
			} else {
				num_of_unclaimed_payouts
			};

			// Get all payout extrinsic for the unclaimed era that fit in the current batch.
			let mut payout_calls = vec![];
			let mut i = 0;
			while i < tx_limit_in_current_batch {
				let payout_era_index = last_reward_received_at_era + i;
				let is_grace_period_satisfied = !grace_period.enabled
					|| (current_era_index - payout_era_index > grace_period.eras);

				let exposure: Exposure<AccountId32, u128> = match api
					.get_storage_double_map(
						"Staking",
						"ErasStakers",
						payout_era_index,
						&validator_account,
						None,
					)
					.await
					.unwrap()
				{
					Some(exposure) => exposure,
					None => Exposure { total: 0, own: 0, others: vec![] },
				};

				if exposure.total.to_be_bytes() > 0_u128.to_be_bytes() && is_grace_period_satisfied
				{
					let payout_extrinsic = api
						.payout_stakers(payout_era_index, validator_account.clone())
						.await
						.unwrap();
					payout_calls.push(payout_extrinsic.function);
				}
				i += 1;
				last_reward_received_at_era += 1;
			}
			num_of_claimed_payouts += payout_calls.len();
			num_of_unclaimed_payouts -= tx_limit_in_current_batch;
			let batch_xt = api.batch(payout_calls).await.unwrap();

			let report =
				api.submit_and_watch_extrinsic_until(batch_xt, XtStatus::InBlock).await.unwrap();
			results.push(format!("{report:?}"));
		}
		println!("{:?}", results);
		println!("Unclaimed payouts: {:?}", num_of_claimed_payouts);
	};
}

pub async fn get_last_reward_received_for(
	account: &AccountId32,
	current_era: EraIndex,
	api: &substrate_api_client::Api<RococoRuntimeConfig, JsonrpseeClient>,
) -> Option<u32> {
	let ledger_storage_key = api.metadata().storage_map_key("Staking", "Ledger", account).unwrap();

	let claimed_rewards: Vec<u32> =
		match api.get_storage_by_key::<StakingLedger>(ledger_storage_key, None).await {
			Ok(Some(ledger)) => ledger.claimed_rewards,
			_ => Vec::new(),
		};

	// Get the era index the last reward has been retrieved.
	let last_reward_received_at_era = if claimed_rewards.is_empty() {
		let history_depth: u32 = api.get_constant("Staking", "HistoryDepth").await.unwrap();
		// Ensure we don't get below zero here.
		if current_era > history_depth {
			let last_known_era = current_era - history_depth;
			Some(last_known_era)
		} else {
			None // The caller most likely has never received any rewards yet.
		}
	} else {
		claimed_rewards.last().copied()
	};
	if let Some(reward_retrieved) = last_reward_received_at_era {
		println!("Retrieved the last reward at era Index {reward_retrieved:?}");
	} else {
		println!("{account:?} did not receive any rewards yet");
	}

	last_reward_received_at_era
}
