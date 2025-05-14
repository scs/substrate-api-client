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

use sp_core::{
	crypto::{Pair, Ss58Codec},
	sr25519,
};
use sp_runtime::MultiAddress;
use substrate_api_client::{
	ac_primitives::RococoRuntimeConfig, extrinsic::BalancesExtrinsics, rpc::TungsteniteRpcClient,
	Api, GetAccountInformation, SubmitAndWatch, XtStatus,
};

fn main() {
	// Setup
	let alice: sr25519::Pair = Pair::from_string(
		"0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
		None,
	)
	.unwrap();
	let client = TungsteniteRpcClient::with_default_url(100);
	let mut api = Api::<RococoRuntimeConfig, _>::new(client).unwrap();
	api.set_signer(alice.clone().into());

	let bob = sr25519::Public::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")
		.unwrap();
	let bob_balance = api.get_account_data(&bob.into()).unwrap().unwrap_or_default().free;

	// Check for failed extrinsic failed onchain
	let xt = api
		.balance_transfer_allow_death(MultiAddress::Id(bob.into()), bob_balance + 1)
		.unwrap();
	let result = api.submit_and_watch_extrinsic_until(xt.clone(), XtStatus::InBlock);
	assert!(format!("{result:?}").contains("FundsUnavailable"));

	// Check directly failed extrinsic (before actually submitted to a block)
	let result = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock);
	assert!(result.is_err());
	assert!(format!("{result:?}").contains("ExtrinsicFailed"));

	// Check for successful extrinisc
	let xt = api
		.balance_transfer_allow_death(MultiAddress::Id(bob.into()), bob_balance / 2)
		.unwrap();
	let _block_hash = api
		.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock)
		.unwrap()
		.block_hash
		.unwrap();
	let bob_new_balance = api.get_account_data(&bob.into()).unwrap().unwrap().free;
	assert!(bob_new_balance > bob_balance);
}
