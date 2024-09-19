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

//! This example shows two special ways to create an extrinsic:
//! - Compose an extrinsic in a no_std environment that works without having acces to an `Api` instance
//! - Compose an extrinsic without asking the node for nonce and without knowing the metadata

use codec::Compact;
use rococo_runtime::{Address, BalancesCall, RuntimeCall};
use sp_keyring::AccountKeyring;
use sp_runtime::{generic::Era, MultiAddress};
use substrate_api_client::{
	ac_compose_macros::{compose_call, compose_extrinsic_offline},
	ac_primitives::{
		config::Config, ExtrinsicParams, ExtrinsicSigner, GenericAdditionalParams, PlainTip,
		RococoRuntimeConfig, SignExtrinsic,
	},
	rpc::JsonrpseeClient,
	Api, GetChainInfo, SubmitAndWatch, XtStatus,
};

type DefaultExtrinsicSigner = <RococoRuntimeConfig as Config>::ExtrinsicSigner;
type AccountId = <RococoRuntimeConfig as Config>::AccountId;
type ExtrinsicAddressOf<Signer> = <Signer as SignExtrinsic<AccountId>>::ExtrinsicAddress;

type Hash = <RococoRuntimeConfig as Config>::Hash;
/// Get the balance type from your node runtime and adapt it if necessary.
type Balance = <RococoRuntimeConfig as Config>::Balance;
type AdditionalParams = GenericAdditionalParams<PlainTip<Balance>, Hash>;

// To test this example with CI we run it against the Polkadot Rococo node. Remember to switch the Config to match your
// own runtime if it uses different parameter configurations. Several pre-compiled runtimes are available in the ac-primitives crate.

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let signer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().await.unwrap();

	let mut api = Api::<RococoRuntimeConfig, _>::new(client).await.unwrap();
	let extrinsic_signer = ExtrinsicSigner::<RococoRuntimeConfig>::new(signer);
	// Signer is needed to set the nonce and sign the extrinsic.
	api.set_signer(extrinsic_signer.clone());

	let recipient: Address = MultiAddress::Id(AccountKeyring::Bob.to_account_id());

	// Get the last finalized header to retrieve information for Era for mortal transactions (online).
	let last_finalized_header_hash = api.get_finalized_head().await.unwrap().unwrap();
	let header = api.get_header(Some(last_finalized_header_hash)).await.unwrap().unwrap();
	let period = 5;

	// Construct extrinsic params needed for the extrinsic construction. For more information on what these parameters mean, take a look at Substrate docs: https://docs.substrate.io/reference/transaction-format/.
	let additional_extrinsic_params: AdditionalParams = GenericAdditionalParams::new()
		.era(Era::mortal(period, header.number.into()), last_finalized_header_hash)
		.tip(0);

	println!("Compose extrinsic in no_std environment (No Api instance)");
	// Get information out of Api (online). This information could also be set offline in the `no_std`,
	// but that would need to be static and adapted whenever the node changes.
	// You can get the information directly from the node runtime file or the api of https://polkadot.js.org.
	let spec_version = api.runtime_version().spec_version;
	let transaction_version = api.runtime_version().transaction_version;
	let genesis_hash = api.genesis_hash();
	let metadata = api.metadata();
	let signer_nonce = api.get_nonce().await.unwrap();
	println!("[+] Alice's Account Nonce is {}", signer_nonce);

	let recipients_extrinsic_address: ExtrinsicAddressOf<DefaultExtrinsicSigner> =
		recipient.clone();

	// Construct an extrinsic using only functionality available in no_std
	let xt = {
		let extrinsic_params = <RococoRuntimeConfig as Config>::ExtrinsicParams::new(
			spec_version,
			transaction_version,
			signer_nonce,
			genesis_hash,
			additional_extrinsic_params,
		);

		let call = compose_call!(
			metadata,
			"Balances",
			"transfer_allow_death",
			recipients_extrinsic_address,
			Compact(4u32)
		)
		.unwrap();
		compose_extrinsic_offline!(extrinsic_signer, call, extrinsic_params)
	};

	println!("[+] Composed Extrinsic:\n {:?}", xt);
	// To send the extrinsic to the node, we need an rpc client which is only available within std-environment. If you want to operate a rpc client in your own no-std environment, take a look at https://github.com/scs/substrate-api-client#rpc-client on how to implement one yourself.
	let hash = api
		.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock)
		.await
		.unwrap()
		.block_hash
		.unwrap();
	println!("[+] Extrinsic got included in block {:?}", hash);

	println!();

	println!("Compose extrinsic offline");
	let signer_nonce = api.get_nonce().await.unwrap();
	println!("[+] Alice's Account Nonce is {}", signer_nonce);

	// Construct an extrinsic offline (without any calls to the node) with the help of the api client. For example, this allows you to set your own nonce (to achieve future calls or construct an extrsinic that must be sent at a later time).
	let xt = {
		// Set the additional params.
		api.set_additional_params(additional_extrinsic_params);

		// Compose the extrinsic (offline).
		let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
			dest: recipient,
			value: 42,
		});
		api.compose_extrinsic_offline(call, signer_nonce)
	};

	println!("[+] Composed Extrinsic:\n {:?}", xt);
	let hash = api
		.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock)
		.await
		.unwrap()
		.block_hash
		.unwrap();
	println!("[+] Extrinsic got included in block {:?}", hash);
}
