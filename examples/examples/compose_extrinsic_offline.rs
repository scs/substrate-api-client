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

//! This examples shows how to use the compose_extrinsic_offline macro which generates an extrinsic
//! without asking the node for nonce and does not need to know the metadata

use ac_primitives::AssetTipExtrinsicParamsBuilder;
use kitchensink_runtime::{BalancesCall, Header, Runtime, RuntimeCall};
use sp_keyring::AccountKeyring;
use sp_runtime::{generic::Era, MultiAddress};

#[cfg(feature = "ws-client")]
use substrate_api_client::rpc::WsRpcClient;

#[cfg(feature = "tungstenite-client")]
use substrate_api_client::rpc::TungsteniteRpcClient;

use substrate_api_client::{
	compose_extrinsic_offline, Api, AssetTipExtrinsicParams, UncheckedExtrinsicV4, XtStatus,
};

fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let from = AccountKeyring::Alice.pair();

	#[cfg(feature = "ws-client")]
	let client = WsRpcClient::with_default_url();

	#[cfg(feature = "tungstenite-client")]
	let client = TungsteniteRpcClient::with_default_url(100);

	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(from);

	// Information for Era for mortal transactions.
	let head = api.get_finalized_head().unwrap().unwrap();
	let h: Header = api.get_header(Some(head)).unwrap().unwrap();
	let period = 5;
	let tx_params = AssetTipExtrinsicParamsBuilder::<Runtime>::new()
		.era(Era::mortal(period, h.number.into()), head)
		.tip(0);

	// Set the custom params builder:
	api.set_extrinsic_params_builder(tx_params);

	// Get the nonce of Alice.
	let alice_nonce = api.get_nonce().unwrap();
	println!("[+] Alice's Account Nonce is {}\n", alice_nonce);

	// Define the recipient.
	let to = MultiAddress::Id(AccountKeyring::Bob.to_account_id());

	// Compose the extrinsic.
	#[allow(clippy::redundant_clone)]
	let xt: UncheckedExtrinsicV4<_, _> = compose_extrinsic_offline!(
		api.signer().unwrap().clone(),
		RuntimeCall::Balances(BalancesCall::transfer { dest: to.clone(), value: 42 }),
		api.extrinsic_params(alice_nonce)
	);

	println!("[+] Composed Extrinsic:\n {:?}\n", xt);

	// Send and watch extrinsic until in block.
	let block_hash = api
		.submit_and_watch_extrinsic_until(&xt.hex_encode(), XtStatus::InBlock)
		.unwrap()
		.unwrap();
	println!("[+] Transaction got included in block {:?}", block_hash);
}
