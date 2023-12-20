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

//! This example shows how to use the compose_extrinsic macro to create an extrinsic for any (custom)
//! module, whereas the desired module and call are supplied as a string.

use codec::Compact;
use kitchensink_runtime::AccountId;
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	ac_compose_macros::{compose_call, compose_extrinsic},
	ac_primitives::{
		AssetRuntimeConfig, ExtrinsicSigner as GenericExtrinsicSigner, SignExtrinsic,
		UncheckedExtrinsicV4,
	},
	rpc::JsonrpseeClient,
	Api, GetAccountInformation, SubmitAndWatch, XtStatus,
};

// To test this example with CI we run it against the Substrate kitchensink node, which uses the asset pallet.
// Therefore, we need to use the `AssetRuntimeConfig` in this example.
// ! However, most Substrate runtimes do not use the asset pallet at all. So if you run an example against your own node
// you most likely should use `DefaultRuntimeConfig` instead.

// Define an extrinsic signer type which sets the generic types of the `GenericExtrinsicSigner`.
// This way, the types don't have to be reassigned with every usage of this type and makes
// the code better readable.
type ExtrinsicSigner = GenericExtrinsicSigner<AssetRuntimeConfig>;

// To access the ExtrinsicAddress type of the Signer, we need to do this via the trait `SignExtrinsic`.
// For better code readability, we define a simple type here and, at the same time, assign the
// AccountId type of the `SignExtrinsic` trait.
type ExtrinsicAddressOf<Signer> = <Signer as SignExtrinsic<AccountId>>::ExtrinsicAddress;

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let sudoer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().await.unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).await.unwrap();
	api.set_signer(sudoer.into());

	// Set the recipient of newly issued funds.
	let recipient = AccountKeyring::Bob.to_account_id();

	// Get the current balance of the recipient.
	let recipient_balance = api.get_account_data(&recipient).await.unwrap().unwrap().free;
	println!("[+] Recipients's Free Balance is now {}\n", recipient_balance);

	// Compose a call that should only be executable via Sudo.
	let recipients_extrinsic_address: ExtrinsicAddressOf<ExtrinsicSigner> =
		recipient.clone().into();
	let new_balance = recipient_balance + 100;
	let call = compose_call!(
		api.metadata(),
		"Balances",
		"force_set_balance",
		recipients_extrinsic_address,
		Compact(new_balance)
	);

	let xt: UncheckedExtrinsicV4<_, _, _, _> = compose_extrinsic!(&api, "Sudo", "sudo", call);

	// Send and watch extrinsic until in block.
	let block_hash = api
		.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock)
		.await
		.unwrap()
		.block_hash
		.unwrap();
	println!("[+] Extrinsic got included. Block Hash: {:?}", block_hash);

	// Ensure the extrinsic has been executed.
	let recipient_new_balance = api.get_account_data(&recipient).await.unwrap().unwrap().free;
	assert_eq!(recipient_new_balance, new_balance);
}
