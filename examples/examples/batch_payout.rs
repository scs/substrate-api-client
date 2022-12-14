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
use kitchensink_runtime::Runtime;
use pallet_staking::{ActiveEraInfo, Exposure};
use serde_json::Value;
use sp_keyring::AccountKeyring;
use sp_runtime::{app_crypto::Ss58Codec, AccountId32};
use substrate_api_client::{
	rpc::JsonrpseeClient, Api, GetStorage, PlainTipExtrinsicParams, SubmitAndWatch, XtStatus,
};

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

	let from = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<_, _, PlainTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(from);
	let grace_period: GracePeriod = GracePeriod { enabled: false, eras: 0 };
	let mut results: Vec<Value> = Vec::new();

	// Give a valid validator account address, given one is westend chain validator account
	let account =
		match AccountId32::from_ss58check("5DJcEbkNxsnNwHGrseg7cgbfUG8eiKzpuZqgSph5HqHrjgf6") {
			Ok(address) => address,
			Err(e) => panic!("Invalid Account id : {:?}", e),
		};

	let active_era: ActiveEraInfo =
		api.get_storage_value("Staking", "ActiveEra", None).unwrap().unwrap();
	let mut last_reward = get_last_reward("5DJcEbkNxsnNwHGrseg7cgbfUG8eiKzpuZqgSph5HqHrjgf6", &api);
	let max_batched_transactions = 9;
	let current_active_era = active_era.index;
	let mut num_of_unclaimed_payout = current_active_era - last_reward - 1;
	let mut start = 1;
	let mut num_of_claimed_payouts = 0;

	while num_of_unclaimed_payout > 0 {
		let mut payout_calls = vec![];
		let mut tx_limit = num_of_unclaimed_payout;
		if num_of_unclaimed_payout > max_batched_transactions {
			tx_limit = max_batched_transactions;
		}

		let mut i = start;
		while i < tx_limit + start {
			let idx = last_reward + i;
			let is_grace_period_satisfied =
				!grace_period.enabled || (current_active_era - idx > grace_period.eras);
			let mut exposure: Exposure<AccountId32, u128> =
				Exposure { total: 0, own: 0, others: vec![] };

			if let Some(exp) = api
				.get_storage_double_map("Staking", "ErasStakers", idx, &account, None)
				.unwrap()
			{
				exposure = exp
			}
			if exposure.total.to_be_bytes() > 0_u128.to_be_bytes() && is_grace_period_satisfied {
				let some = api.payout_stakers(idx, account.clone());
				payout_calls.push(some.function);
			}
			i += 1;
			last_reward += 1;
		}
		let payout_calls_len = payout_calls.len();
		if payout_calls_len > 0 {
			let batching = api.batch(payout_calls);
			let results_hash = api
				.submit_and_watch_extrinsic_until(&batching.hex_encode(), XtStatus::InBlock)
				.unwrap();
			num_of_claimed_payouts += payout_calls_len;

			let result = serde_json::to_value(results_hash).unwrap();
			results.push(result);
		}
		num_of_unclaimed_payout -= tx_limit;
		start += tx_limit;
	}
	println!("{:?}", results);
	println!("Unclaimed payouts: {:?}", num_of_claimed_payouts);
}

pub fn get_last_reward(
	validator_address: &str,
	api: &substrate_api_client::Api<
		sp_core::sr25519::Pair,
		JsonrpseeClient,
		PlainTipExtrinsicParams<Runtime>,
		Runtime,
	>,
) -> u32 {
	let account = match AccountId32::from_ss58check(validator_address) {
		Ok(address) => address,
		Err(e) => panic!("Invalid Account id : {:?}", e),
	};
	let active_era: ActiveEraInfo =
		api.get_storage_value("Staking", "ActiveEra", None).unwrap().unwrap();
	let storagekey = api.metadata().storage_map_key("Staking", "Ledger", &account).unwrap();
	let mut res = StakingLedger {
		stash: account,
		total: 0,
		active: 0,
		unlocking: Vec::new(),
		claimed_rewards: Vec::new(),
	};

	if let Ok(Some(ledger)) = api.get_storage_by_key_hash(storagekey, None) {
		res = ledger
	}

	let is_history_checked_force: bool = false;

	let last_reward = if is_history_checked_force || res.claimed_rewards.is_empty() {
		let history_depth: u32 = api.get_constant("Staking", "HistoryDepth").unwrap();
		active_era.index - history_depth
	} else {
		res.claimed_rewards.pop().unwrap()
	};
	println!("{}", last_reward);
	last_reward
}
