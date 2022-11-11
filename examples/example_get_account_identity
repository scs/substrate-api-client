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

///! Example to show how to get the account identity display name from the identity pallet.
use clap::{load_yaml, App};
use frame_metadata::StorageEntryType;
use kitchensink_runtime::Runtime as KitchensinkRuntime;
use pallet_identity::types::Registration;
use scale_info::form::PortableForm;
use sp_keyring::AccountKeyring;
use substrate_api_client::{rpc::WsRpcClient, AccountInfo, Api, AssetTipExtrinsicParams};

fn main() {
	env_logger::init();
	let url = get_node_url_from_cli();

	let client = WsRpcClient::new(&url);
	let mut api = Api::<_, _, AssetTipExtrinsicParams>::new(client).unwrap();
	let account = AccountKeyring::Alice.public();

	// Get the storage value from the pallet. Check out the pallet itself to know it's type:
	// see https://github.com/paritytech/substrate/blob/e6768a3bd553ddbed12fe1a0e4a2ef8d4f8fdf52/frame/identity/src/lib.rs#L167
	type RegistrationType = Registration<
		BalanceOf<KitchensinkRuntime>,
		KitchensinkRuntime::MaxRegistrars,
		KitchensinkRuntime::MaxAdditionalFields,
	>;

	let registration: RegistrationType =
		api.get_storage_map("Identity", "IdentityOf", account, None).unwrap().unwrap();
	println!("[+] Registration is {}", registration);
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
