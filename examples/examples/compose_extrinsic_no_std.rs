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

//! This example shows how to use the compose_extrinsic_offline macro which generates an extrinsic
//! without asking the node for nonce and does not need to know the metadata

use codec::Compact;
use kitchensink_runtime::{BalancesCall, RuntimeCall};
use sp_keyring::AccountKeyring;
use sp_runtime::{generic::Era, MultiAddress};
use substrate_api_client::{
	ac_compose_macros::{compose_call, compose_extrinsic_offline},
	ac_primitives::{
		config::Config, AssetTip, ExtrinsicParams, ExtrinsicSigner, GenericAdditionalParams,
		SignExtrinsic, SubstrateKitchensinkConfig,
	},
	rpc::JsonrpseeClient,
	Api, GetChainInfo, SubmitAndWatch, XtStatus,
};

type KitchensinkExtrinsicSigner = <SubstrateKitchensinkConfig as Config>::ExtrinsicSigner;
type AccountId = <SubstrateKitchensinkConfig as Config>::AccountId;
type ExtrinsicAddressOf<Signer> = <Signer as SignExtrinsic<AccountId>>::ExtrinsicAddress;

type Hash = <SubstrateKitchensinkConfig as Config>::Hash;
/// Get the balance type from your node runtime and adapt it if necessary.
type Balance = <SubstrateKitchensinkConfig as Config>::Balance;
/// We need AssetTip here, because the kitchensink runtime uses the asset pallet. Change to PlainTip if your node uses the balance pallet only.
type AdditionalParams = GenericAdditionalParams<AssetTip<Balance>, Hash>;

type Address = <SubstrateKitchensinkConfig as Config>::Address;

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize api and set the signer (sender) that is used to sign the extrinsics.
	let signer = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().unwrap();
	// Api::new(..) is not actually an offline call, but retrieves metadata and other information from the node.
	// If this is not acceptable, use the Api::new_offline(..) function instead. There are no examples for this,
	// because of the constantly changing substrate node. But check out our unit tests - there are Apis created with `new_offline`.
	//
	// ! Careful: AssetTipExtrinsicParams is used here, because the substrate kitchensink runtime uses assets as tips. But for most
	// runtimes, the PlainTipExtrinsicParams needs to be used.
	let mut api = Api::<SubstrateKitchensinkConfig, _>::new(client).unwrap();
	let extrinsic_signer = ExtrinsicSigner::<_>::new(signer);
	// Signer is needed to set the nonce and sign the extrinsic.
	api.set_signer(extrinsic_signer.clone());

	let recipient: Address = MultiAddress::Id(AccountKeyring::Bob.to_account_id());

	// Get the last finalized header to retrieve information for Era for mortal transactions (online).
	let last_finalized_header_hash = api.get_finalized_head().unwrap().unwrap();
	let header = api.get_header(Some(last_finalized_header_hash)).unwrap().unwrap();
	let period = 5;

	// Construct extrinsic without using Api (no_std).
	let additional_extrinsic_params: AdditionalParams = GenericAdditionalParams::new()
		.era(Era::mortal(period, header.number.into()), last_finalized_header_hash)
		.tip(0);

	let signer_nonce = api.get_nonce().unwrap();
	println!("[+] Alice's Account Nonce is {}\n", signer_nonce);

	let use_no_std = true;
	let hash = if use_no_std {
		// Get information out of Api (online). This information could also be set offline in the `no_std`,
		// but that would need to be static and adapted whenever the node changes.
		// You can get the information directly from the node runtime file or the api of https://polkadot.js.org.
		let spec_version = api.runtime_version().spec_version;
		let transaction_version = api.runtime_version().transaction_version;
		let genesis_hash = api.genesis_hash();
		let metadata = api.metadata();

		let extrinsic_params = <SubstrateKitchensinkConfig as Config>::ExtrinsicParams::new(
			spec_version,
			transaction_version,
			signer_nonce,
			genesis_hash,
			additional_extrinsic_params,
		);

		let recipients_extrinsic_address: ExtrinsicAddressOf<KitchensinkExtrinsicSigner> =
			recipient.clone().into();

		let call = compose_call!(
			metadata,
			"Balances",
			"transfer_allow_death",
			recipients_extrinsic_address,
			Compact(4u32)
		);
		let xt = compose_extrinsic_offline!(extrinsic_signer, call, extrinsic_params);
		println!("[+] Composed Extrinsic:\n {:?}\n", xt);

		api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock)
			.unwrap()
			.block_hash
			.unwrap()
	} else {
		// Set the additional params.
		api.set_additional_params(additional_extrinsic_params);

		// Compose the extrinsic (offline).
		let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
			dest: recipient,
			value: 42,
		});
		let xt = api.compose_extrinsic_offline(call, signer_nonce);
		println!("[+] Composed Extrinsic:\n {:?}\n", xt);

		api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock)
			.unwrap()
			.block_hash
			.unwrap()
	};
	println!("[+] Extrinsic got included in block {:?}", hash);
}
