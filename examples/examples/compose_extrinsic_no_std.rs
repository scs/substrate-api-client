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
use kitchensink_runtime::{AccountId, BalancesCall, RuntimeCall};
use sp_core::H256;
use sp_keyring::AccountKeyring;
use sp_runtime::{generic::Era, AccountId32, MultiAddress};
use substrate_api_client::{
	ac_compose_macros::{compose_call, compose_extrinsic_offline},
	ac_primitives::{
		ExtrinsicParams, ExtrinsicSigner, GenericAdditionalParams, GenericExtrinsicParams,
		PlainTip, SignExtrinsic, SubstrateKitchensinkConfig,
	},
	rpc::JsonrpseeClient,
	Api, GetChainInfo, SubmitExtrinsic,
};

type KitchensinkExtrinsicSigner = ExtrinsicSigner<SubstrateKitchensinkConfig>;
type ExtrinsicAddressOf<Signer> = <Signer as SignExtrinsic<AccountId>>::ExtrinsicAddress;

type Hash = H256; //<Runtime as FrameSystemConfig>::Hash;
/// Get the balance type from your node runtime and adapt it if necessary.
type Balance = u128;
/// We need AssetTip here, because the kitchensink runtime uses the asset pallet. Change to PlainTip if your node uses the balance pallet only.
type AdditionalParams = GenericAdditionalParams<PlainTip<Balance>, Hash>;

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
	// Signer is needed to get the nonce
	api.set_signer(extrinsic_signer.clone());

	// Information for Era for mortal transactions (online).
	let last_finalized_header_hash = api.get_finalized_head().unwrap().unwrap();
	let header = api.get_header(Some(last_finalized_header_hash)).unwrap().unwrap();
	let period = 5;

	// Get information out of Api (online).
	let spec_version = api.runtime_version().spec_version;
	let transaction_version = api.runtime_version().transaction_version;
	let genesis_hash = api.genesis_hash();
	let signer_nonce = api.get_nonce().unwrap();
	println!("[+] Alice's Account Nonce is {}\n", signer_nonce);

	let recipient: MultiAddress<AccountId32, u32> =
		MultiAddress::Id(AccountKeyring::Bob.to_account_id());
	let recipient_new = AccountKeyring::Bob.to_account_id();
	//let recipient: MultiAddress<AccountId32, u32> =
	//	MultiAddress::Id(AccountKeyring::Bob.to_account_id());

	// Construct extrinsic without using Api (no_std).
	let additional_extrinsic_params: AdditionalParams = GenericAdditionalParams::new()
		.era(Era::mortal(period, header.number.into()), last_finalized_header_hash)
		.tip(0);
	let extrinsic_params =
		GenericExtrinsicParams::<SubstrateKitchensinkConfig, PlainTip<u128>>::new(
			spec_version,
			transaction_version,
			signer_nonce,
			genesis_hash,
			additional_extrinsic_params,
		);
	//let pallet = api.metadata().pallet("balances").unwrap();
	//let pallet_index = pallet.index();
	//let call_index = pallet.call_index("transfer_allow_death").unwrap();
	let call =
		RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: recipient, value: 42 });

	let recipients_extrinsic_address: ExtrinsicAddressOf<KitchensinkExtrinsicSigner> =
		recipient_new.clone().into();

	let call_new = compose_call!(
		api.metadata(),
		"Balances",
		"transfer_allow_death",
		recipients_extrinsic_address,
		Compact(4u32)
	);
	let xt_no_std = compose_extrinsic_offline!(extrinsic_signer, call_new.clone(), extrinsic_params);
	println!("[+] Composed Extrinsic:\n {:?}\n", xt_no_std);

	// Submit extrinsic (online)
	let hash = api.submit_extrinsic(xt_no_std);
	println!("[+] Extrinsic got included in block {:?}", hash);
}
