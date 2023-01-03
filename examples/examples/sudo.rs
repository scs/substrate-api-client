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

use codec::{Compact, Encode};
use kitchensink_runtime::Runtime;
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	compose_call, compose_extrinsic, rpc::JsonrpseeClient, Api, AssetTipExtrinsicParams,
	GenericAddress, GetAccountInformation, SubmitAndWatch, UncheckedExtrinsicV4, XtStatus,
};

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let sudoer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(sudoer);

	// Set the recipient of newly issued funds.
	let recipient = AccountKeyring::Bob.to_account_id();

	// Get the current balance of the recipient.
	let recipient_balance = api.get_account_data(&recipient).unwrap().unwrap().free;
	println!("[+] Recipients's Free Balance is now {}\n", recipient_balance);

	// Compose a call that should only be executable via Sudo.
	let new_balance = recipient_balance + 100;
	let call = compose_call!(
		api.metadata(),
		"Balances",
		"set_balance",
		GenericAddress::Id(recipient.clone()),
		Compact(new_balance),
		Compact(new_balance)
	);

	let xt: UncheckedExtrinsicV4<_, _> = compose_extrinsic!(&api, "Sudo", "sudo", call);

	// Send and watch extrinsic until in block.
	let block_hash = api
		.submit_and_watch_extrinsic_until(xt.encode(), XtStatus::InBlock)
		.unwrap()
		.block_hash
		.unwrap();
	println!("[+] Extrinsic got included. Block Hash: {:?}", block_hash);

	// Ensure the extrinisc has been executed.
	let recipient_new_balance = api.get_account_data(&recipient).unwrap().unwrap().free;
	assert_eq!(recipient_new_balance, new_balance);
}
