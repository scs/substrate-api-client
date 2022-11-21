#[cfg(feature = "staking-xt")]
use clap::{load_yaml, App};
#[cfg(feature = "staking-xt")]
use pallet_staking::{ActiveEraInfo, Exposure};
#[cfg(feature = "staking-xt")]
use sp_keyring::AccountKeyring;
#[cfg(feature = "staking-xt")]
use sp_runtime::{app_crypto::Ss58Codec, AccountId32};
#[cfg(feature = "staking-xt")]
use substrate_api_client::{rpc::WsRpcClient, Api, PlainTipExtrinsicParams, XtStatus};

#[cfg(feature = "staking-xt")]
fn main() {
	env_logger::init();
	let url = get_node_url_from_cli();
	let from = AccountKeyring::Alice.pair();
	let client = WsRpcClient::new(&url);
	let api = Api::<_, _, PlainTipExtrinsicParams>::new(client)
		.map(|api| api.set_signer(from))
		.unwrap();
	let mut exposure: Exposure<AccountId32, u128> = Exposure { total: 0, own: 0, others: vec![] };
	let account =
		match AccountId32::from_ss58check("5DJcEbkNxsnNwHGrseg7cgbfUG8eiKzpuZqgSph5HqHrjgf6") {
			Ok(address) => address,
			Err(e) => panic!("Invalid Account id : {:?}", e),
		};
	let active_era: ActiveEraInfo =
		api.get_storage_value("Staking", "ActiveEra", None).unwrap().unwrap();
	println!("{:?}", active_era);
	let idx = active_era.index - 1;
	if let Ok(Some(exp)) = api.get_storage_double_map("Staking", "ErasStakers", idx, &account, None)
	{
		exposure = exp;
	}
	if exposure.total > 0_u128 {
		let call = api.payout_stakers(idx, account);
		let result = api.send_extrinsic(call.hex_encode(), XtStatus::InBlock).unwrap();
		println!("{:?}", result);
	}
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

#[cfg(not(feature = "staking-xt"))]
fn main() {}
