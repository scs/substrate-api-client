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

use codec::Encode;
use kitchensink_runtime::Runtime;
use pallet_staking::{ActiveEraInfo, Exposure};
use sp_keyring::AccountKeyring;
use sp_runtime::{app_crypto::Ss58Codec, AccountId32};
use substrate_api_client::{
	rpc::JsonrpseeClient, Api, AssetTipExtrinsicParams, GetStorage, SubmitAndWatch, XtStatus,
};

#[tokio::main]
async fn main() {
	env_logger::init();
	let from = AccountKeyring::Alice.pair();
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<_, _, AssetTipExtrinsicParams<Runtime>, Runtime>::new(client).unwrap();
	api.set_signer(from);
	let mut exposure: Exposure<AccountId32, u128> = Exposure { total: 0, own: 0, others: vec![] };
	let account =
		match AccountId32::from_ss58check("5DJcEbkNxsnNwHGrseg7cgbfUG8eiKzpuZqgSph5HqHrjgf6") {
			Ok(address) => address,
			Err(e) => panic!("Invalid Account id : {:?}", e),
		};
	let active_era: ActiveEraInfo =
		api.get_storage_value("Staking", "ActiveEra", None).unwrap().unwrap();
	println!("{:?}", active_era);
	let idx = active_era.index - 1;
	if let Ok(Some(exp)) = api.get_storage_double_map("Staking", "ErasStakers", idx, &account, None)
	{
		exposure = exp;
	}
	if exposure.total > 0_u128 {
		let call = api.payout_stakers(idx, account);
		let report =
			api.submit_and_watch_extrinsic_until(call.encode(), XtStatus::InBlock).unwrap();
		println!("{:?}", report);
	}
}
