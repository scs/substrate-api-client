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

//! Very simple example that shows how to subscribe to events generically
//! implying no runtime needs to be imported

use codec::Decode;
use kitchensink_runtime::Runtime;
use sp_keyring::AccountKeyring;
use sp_runtime::{AccountId32 as AccountId, MultiAddress};
use std::thread;
use substrate_api_client::{
	rpc::JsonrpseeClient, Api, AssetTipExtrinsicParams, StaticEvent, SubmitAndWatch,
	SubscribeEvents, SubscribeFrameSystem, XtStatus,
};

// Look at the how the transfer event looks like in in the metadata
#[derive(Decode)]
struct TransferEventArgs {
	from: AccountId,
	to: AccountId,
	value: u128,
}

impl StaticEvent for TransferEventArgs {
	const PALLET: &'static str = "Balances";
	const EVENT: &'static str = "Transfer";
}

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let alice = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(alice);

	println!("Subscribe to events");

	let api2 = api.clone();
	let thread_output = thread::spawn(move || {
		let mut subscription = api2.subscribe_system_events().unwrap();
		let args: TransferEventArgs =
			api2.wait_for_event::<TransferEventArgs>(&mut subscription).unwrap();
		args
	});

	// Bob
	let bob = AccountKeyring::Bob.to_account_id();

	// Generate extrinsic.
	let xt = api.balance_transfer(MultiAddress::Id(bob), 1000000000000);
	println!("[+] Composed extrinsic: {:?}\n", xt);

	// Send extrinsic.
	let tx_hash = api
		.submit_and_watch_extrinsic_until(&xt.hex_encode(), XtStatus::InBlock)
		.unwrap()
		.unwrap();
	println!("[+] Transaction got included. Hash: {:?}\n", tx_hash);

	let args = thread_output.join().unwrap();

	println!("Transactor: {:?}", args.from);
	println!("Destination: {:?}", args.to);
	println!("Value: {:?}", args.value);
}
