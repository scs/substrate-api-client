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
use kitchensink_runtime::{Runtime as KitchensinkRuntime, Signature};
use pallet_identity::{Data, IdentityInfo, Registration};
use sp_core::{crypto::Pair, H256};
use sp_keyring::AccountKeyring;
use substrate_api_client::{
	ac_compose_macros::compose_extrinsic,
	ac_primitives::{AssetTipExtrinsicParams, ExtrinsicSigner, UncheckedExtrinsicV4},
	rpc::JsonrpseeClient,
	Api, GetStorage, SubmitAndWatch, XtStatus,
};

type BalanceOf<T> = <<T as pallet_identity::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
type MaxRegistrarsOf<T> = <T as pallet_identity::Config>::MaxRegistrars;
type MaxAdditionalFieldsOf<T> = <T as pallet_identity::Config>::MaxAdditionalFields;

#[tokio::main]
async fn main() {
	env_logger::init();

	// Create the node-api client and set the signer.
	let client = JsonrpseeClient::with_default_url().unwrap();
	let signer = AccountKeyring::Alice.pair();
	// ! Careful: AssetTipExtrinsicParams is used here, because the substrate kitchensink runtime uses assets as tips. But for most
	// runtimes, the PlainTipExtrinsicParams needs to be used.
	let mut api =
		Api::<_, _, AssetTipExtrinsicParams<KitchensinkRuntime>, KitchensinkRuntime>::new(client)
			.unwrap();
	api.set_signer(ExtrinsicSigner::<_, Signature, KitchensinkRuntime>::new(signer.clone()));

	// Fill Identity storage.
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
	// see https://github.com/paritytech/substrate/blob/e6768a3bd553ddbed12fe1a0e4a2ef8d4f8fdf52/frame/identity/src/lib.rs#L167
	type RegistrationType = Registration<
		BalanceOf<KitchensinkRuntime>,
		MaxRegistrarsOf<KitchensinkRuntime>,
		MaxAdditionalFieldsOf<KitchensinkRuntime>,
	>;

	let registration: RegistrationType = api
		.get_storage_map("Identity", "IdentityOf", signer.public(), None)
		.unwrap()
		.unwrap();
	println!("[+] Retrieved {:?}", registration);
	assert_eq!(registration.info, info);
}
