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

//! This examples shows how to use the compose_extrinsic macro to create an extrinsic for any (custom)
//! module, whereas the desired module and call are supplied as a string.

use kitchensink_runtime::Runtime;
use sp_keyring::AccountKeyring;

#[cfg(feature = "ws-client")]
use substrate_api_client::rpc::WsRpcClient;

#[cfg(feature = "tungstenite-client")]
use substrate_api_client::rpc::TungsteniteRpcClient;

use substrate_api_client::{
	compose_extrinsic, Api, AssetTipExtrinsicParams, GenericAddress, UncheckedExtrinsicV4, XtStatus,
};

fn main() {
	env_logger::init();

	// initialize api and set the signer (sender) that is used to sign the extrinsics
	let from = AccountKeyring::Alice.pair();

	#[cfg(feature = "ws-client")]
	let client = WsRpcClient::with_default_url();

	#[cfg(feature = "tungstenite-client")]
	let client = TungsteniteRpcClient::with_default_url(100);

	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(from);
	// set the recipient
	let to = AccountKeyring::Bob.to_account_id();

	// call Balances::transfer
	// the names are given as strings
	#[allow(clippy::redundant_clone)]
	let xt: UncheckedExtrinsicV4<_, _> = compose_extrinsic!(
		api.clone(),
		"Balances",
		"transfer",
		GenericAddress::Id(to),
		Compact(42_u128)
	);

	println!("[+] Composed Extrinsic:\n {:?}\n", xt);

	// send and watch extrinsic until InBlock
	let tx_hash = api
		.submit_and_watch_extrinsic_until(&xt.hex_encode(), XtStatus::InBlock)
		.unwrap();
	println!("[+] Transaction got included. Hash: {:?}", tx_hash);
}
