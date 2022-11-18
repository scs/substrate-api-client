#[cfg(feature = "staking-xt")]
use clap::{load_yaml, App};
#[cfg(feature = "staking-xt")]
use codec::{Decode, Encode};
#[cfg(feature = "staking-xt")]
use serde_json::Value;
#[cfg(feature = "staking-xt")]
use sp_core::{sr25519, Pair};
#[cfg(feature = "staking-xt")]
use sp_keyring::AccountKeyring;
#[cfg(feature = "staking-xt")]
use sp_runtime::{app_crypto::Ss58Codec, AccountId32};
#[cfg(feature = "staking-xt")]
use staking::{ActiveEraInfo, Exposure};
#[cfg(feature = "staking-xt")]
use substrate_api_client::{
	rpc::WsRpcClient, Api, BaseExtrinsicParams, PlainTip, PlainTipExtrinsicParams, XtStatus,
};

#[cfg(feature = "staking-xt")]
fn main() {
	env_logger::init();

	let url = get_node_url_from_cli();
	let from = AccountKeyring::Alice.pair();
	let client = WsRpcClient::new(&url);
	let api = Api::<_, _, PlainTipExtrinsicParams>::new(client)
		.map(|api| api.set_signer(from))
		.unwrap();
	let grace_period: GracePeriod = GracePeriod { enabled: false, eras: 0 };
	let mut results: Vec<Value> = Vec::new();
	let account: AccountId32;
	// Give a valid validator account address, given one is westend chain validator account
	match AccountId32::from_ss58check("5DJcEbkNxsnNwHGrseg7cgbfUG8eiKzpuZqgSph5HqHrjgf6") {
		Ok(address) => account = address,
		Err(e) => panic!("Invalid Account id : {:?}", e),
	}

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
		while i <= tx_limit + start - 1 {
			let idx = last_reward + i;
			let is_grace_period_satisfied =
				!grace_period.enabled || (current_active_era - idx > grace_period.eras);
			let mut exposure: Exposure<AccountId32, u128> =
				Exposure { total: 0, own: 0, others: vec![] };

			match api
				.get_storage_double_map("Staking", "ErasStakers", idx, &account, None)
				.unwrap()
			{
				Some(exp) => exposure = exp,
				None => (),
			}
			if exposure.total.to_be_bytes() > 0_u128.to_be_bytes() && is_grace_period_satisfied {
				let some = api.payout_stakers(idx, account.clone());
				payout_calls.push(some.function);
			}
			i += 1;
			last_reward = last_reward + 1;
		}
		let mut current_tx_done = false;
		let mut payout_calls_len = payout_calls.len();
		if payout_calls_len > 0 {
			let batching = api.batch(payout_calls);
			let results_hash =
				api.send_extrinsic(batching.hex_encode(), XtStatus::InBlock).unwrap();
			num_of_claimed_payouts += payout_calls_len;

			let result = serde_json::to_value(results_hash).unwrap();
			results.push(result);
		} else {
			current_tx_done = true;
		}
		num_of_unclaimed_payout -= tx_limit;
		start += tx_limit;
	}
	println!("{:?}", results);
}

#[cfg(feature = "staking-xt")]
pub fn get_node_url_from_cli() -> String {
	let yml = load_yaml!("cli.yml");
	let matches = App::from_yaml(yml).get_matches();

	let node_ip = matches.value_of("node-server").unwrap_or("ws://127.0.0.1");
	let node_port = matches.value_of("node-port").unwrap_or("9944");
	let url = format!("{}:{}", node_ip, node_port);
	println!("Interacting with node on {}\n", url);
	url
}

#[cfg(feature = "staking-xt")]
pub fn get_last_reward(
	validator_address: &str,
	api: &substrate_api_client::Api<
		sp_core::sr25519::Pair,
		WsRpcClient,
		BaseExtrinsicParams<PlainTip>,
	>,
) -> u32 {
	let api = api;
	let mut account: AccountId32;
	match AccountId32::from_ss58check(&validator_address) {
		Ok(address) => account = address,
		Err(e) => panic!("Invalid Account id : {:?}", e),
	}

	let active_era: ActiveEraInfo =
		api.get_storage_value("Staking", "ActiveEra", None).unwrap().unwrap();
	let storagekey = api.metadata.storage_map_key("Staking", "Ledger", &account).unwrap();
	let mut res = StakingLedger {
		stash: account.clone(),
		total: 0,
		active: 0,
		unlocking: Vec::new(),
		claimed_rewards: Vec::new(),
	};

	match api.get_storage_by_key_hash(storagekey, None) {
		Ok(Some(ledger)) => res = ledger,
		_ => (),
	}

	let mut last_reward = 0_u32;
	let is_history_checked_force: bool = false;

	if is_history_checked_force || res.claimed_rewards.len() == 0 {
		last_reward = api.get_constant("Staking", "HistoryDepth").unwrap();
		last_reward = active_era.index - last_reward;
	} else {
		last_reward = res.claimed_rewards.pop().unwrap();
	}
	println!("{}", last_reward);
	last_reward
}
#[cfg(feature = "staking-xt")]
pub struct GracePeriod {
	pub enabled: bool,
	pub eras: u32,
}
#[cfg(feature = "staking-xt")]
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

#[cfg(not(feature = "staking-xt"))]
fn main() {}
