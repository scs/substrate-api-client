use clap::{load_yaml, App};
use sp_core::{sr25519, Pair};
use sp_keyring::AccountKeyring;
use sp_runtime::{app_crypto::Ss58Codec, AccountId32};
use staking::{ActiveEraInfo, Exposure};
use substrate_api_client::{rpc::WsRpcClient, AccountId, Api, PlainTipExtrinsicParams, XtStatus};

fn main() {
	env_logger::init();

	let url = get_node_url_from_cli();

	let from = AccountKeyring::Alice.pair();
	let client = WsRpcClient::new(&url);
	let api = Api::<_, _, PlainTipExtrinsicParams>::new(client)
		.map(|api| api.set_signer(from))
		.unwrap();

	let mut exposure: Exposure<AccountId32, u128> = Exposure { total: 0, own: 0, others: vec![] };

	let account: AccountId;
	match AccountId32::from_ss58check("5DJcEbkNxsnNwHGrseg7cgbfUG8eiKzpuZqgSph5HqHrjgf6") {
		Ok(address) => account = address,
		Err(e) => panic!("Invalid Account id : {:?}", e),
	}

	let active_era: ActiveEraInfo =
		api.get_storage_value("Staking", "ActiveEra", None).unwrap().unwrap();
	println!("{:?}", active_era);
	let idx = active_era.index - 1;
	match api
		.get_storage_double_map("Staking", "ErasStakers", idx, &account, None)
		.unwrap()
	{
		Some(exp) => {
			exposure = exp;
		},
		None => (),
	}
	if exposure.total > 0_u128 {
		let call = api.payout_stakers(idx, account.clone());
		let result = api.send_extrinsic(call.hex_encode(), XtStatus::InBlock).unwrap();
		println!("{:?}", result);
	}
}

pub fn get_node_url_from_cli() -> String {
	let yml = load_yaml!("cli.yml");
	let matches = App::from_yaml(yml).get_matches();

	let node_ip = matches.value_of("node-server").unwrap_or("ws://127.0.0.1");
	let node_port = matches.value_of("node-port").unwrap_or("9944");
	let url = format!("{}:{}", node_ip, node_port);
	println!("Interacting with node on {}\n", url);
	url
}
