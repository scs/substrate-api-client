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
use kitchensink_runtime::{MaxAdditionalFields, Runtime as KitchensinkRuntime};
use pallet_identity::{legacy::IdentityInfo, Data, Registration};
use sp_core::{crypto::Pair, H256};
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	ac_compose_macros::compose_extrinsic,
	ac_primitives::{AssetRuntimeConfig, ExtrinsicSigner, UncheckedExtrinsicV4},
	rpc::JsonrpseeClient,
	Api, GetStorage, SubmitAndWatch, XtStatus,
};

type BalanceOf<T> = <<T as pallet_identity::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
type MaxRegistrarsOf<T> = <T as pallet_identity::Config>::MaxRegistrars;
type IdentityInformation<T> = <T as pallet_identity::Config>::IdentityInformation;

// To test this example with CI we run it against the Substrate kitchensink node, which uses the asset pallet.
// Therefore, we need to use the `AssetRuntimeConfig` in this example.
// ! However, most Substrate runtimes do not use the asset pallet at all. So if you run an example against your own node
// you most likely should use `DefaultRuntimeConfig` instead.

#[tokio::main]
async fn main() {
	env_logger::init();

	// Create the node-api client and set the signer.
	let client = JsonrpseeClient::with_default_url().unwrap();
	let signer = AccountKeyring::Alice.pair();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).unwrap();
	api.set_signer(ExtrinsicSigner::<AssetRuntimeConfig>::new(signer.clone()));

	// Fill Identity storage.
	let info = IdentityInfo::<MaxAdditionalFields> {
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

	let xt: UncheckedExtrinsicV4<_, _, _, _> =
		compose_extrinsic!(&api, "Identity", "set_identity", Box::new(info.clone()));
	println!("[+] Composed Extrinsic:\n {:?}\n", xt);

	// Send and watch extrinsic until InBlock.
	let _block_hash = api
		.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock)
		.unwrap()
		.block_hash
		.unwrap();

	// Get the storage value from the pallet. Check out the pallet itself to know it's type:
	// see https://github.com/paritytech/polkadot-sdk/blob/91851951856b8effe627fb1d151fe336a51eef2d/substrate/frame/identity/src/lib.rs#L170
	type RegistrationType = Registration<
		BalanceOf<KitchensinkRuntime>,
		MaxRegistrarsOf<KitchensinkRuntime>,
		IdentityInformation<KitchensinkRuntime>,
	>;

	let registration: RegistrationType = api
		.get_storage_map("Identity", "IdentityOf", signer.public(), None)
		.unwrap()
		.unwrap();
	println!("[+] Retrieved {:?}", registration);
	assert_eq!(registration.info, info);
}
