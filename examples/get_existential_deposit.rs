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

//! Very simple example that shows how to get some simple storage values.

use kitchensink_runtime::Runtime;
use sp_runtime::app_crypto::sp_core::sr25519;

#[cfg(feature = "ws-client")]
use substrate_api_client::rpc::WsRpcClient;

#[cfg(feature = "tungstenite-client")]
use substrate_api_client::rpc::TungsteniteRpcClient;

use substrate_api_client::{Api, AssetTipExtrinsicParams};

fn main() {
	env_logger::init();

	#[cfg(feature = "ws-client")]
	let client = WsRpcClient::with_default_url();

	#[cfg(feature = "tungstenite-client")]
	let client = TungsteniteRpcClient::with_default_url(100);

	let api =
		Api::<sr25519::Pair, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();

	// get existential deposit
	let min_balance = api.get_existential_deposit().unwrap();
	println!("[+] Existential Deposit is {}", min_balance);
}
