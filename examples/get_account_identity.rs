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

//! Example to show how to get the account identity display name from the identity pallet.

use frame_support::traits::Currency;
use kitchensink_runtime::Runtime as KitchensinkRuntime;
use pallet_identity::{Data, IdentityInfo, Registration};
use sp_core::{crypto::Pair, H256};
use sp_keyring::AccountKeyring;

#[cfg(feature = "ws-client")]
use substrate_api_client::rpc::WsRpcClient;

#[cfg(feature = "tungstenite-client")]
use substrate_api_client::rpc::TungsteniteRpcClient;

use substrate_api_client::{
	compose_extrinsic, Api, AssetTipExtrinsicParams, UncheckedExtrinsicV4, XtStatus,
};

type BalanceOf<T> = <<T as pallet_identity::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
type MaxRegistrarsOf<T> = <T as pallet_identity::Config>::MaxRegistrars;
type MaxAdditionalFieldsOf<T> = <T as pallet_identity::Config>::MaxAdditionalFields;

fn main() {
	env_logger::init();

	// Create the node-api client and set the signer.
	#[cfg(feature = "ws-client")]
	let client = WsRpcClient::new("ws://127.0.0.1:9944");

	#[cfg(feature = "tungstenite-client")]
	let client = TungsteniteRpcClient::new(url::Url::parse("ws://127.0.0.1:9944").unwrap(), 100);

	let alice = AccountKeyring::Alice.pair();
	let mut api =
		Api::<_, _, AssetTipExtrinsicParams<KitchensinkRuntime>, KitchensinkRuntime>::new(client)
			.unwrap();
	api.set_signer(alice.clone());

	// Fill Identity storage
	let info = IdentityInfo::<MaxAdditionalFieldsOf<KitchensinkRuntime>> {
		additional: Default::default(),
		display: Data::Keccak256(H256::random().into()),
		legal: Data::None,
		web: Data::None,
		riot: Data::None,
		email: Data::None,
		pgp_fingerprint: None,
		image: Data::None,
		twitter: Data::None,
	};

	#[allow(clippy::redundant_clone)]
	let xt: UncheckedExtrinsicV4<_, _> =
		compose_extrinsic!(api.clone(), "Identity", "set_identity", Box::new(info.clone()));
	println!("[+] Composed Extrinsic:\n {:?}\n", xt);

	// Send and watch extrinsic until InBlock.
	let _block_hash = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock).unwrap();

	// Get the storage value from the pallet. Check out the pallet itself to know it's type:
	// see https://github.com/paritytech/substrate/blob/e6768a3bd553ddbed12fe1a0e4a2ef8d4f8fdf52/frame/identity/src/lib.rs#L167
	type RegistrationType = Registration<
		BalanceOf<KitchensinkRuntime>,
		MaxRegistrarsOf<KitchensinkRuntime>,
		MaxAdditionalFieldsOf<KitchensinkRuntime>,
	>;

	let registration: RegistrationType = api
		.get_storage_map("Identity", "IdentityOf", alice.public(), None)
		.unwrap()
		.unwrap();
	println!("[+] Retrieved {:?}", registration);
	assert_eq!(registration.info, info);
}
