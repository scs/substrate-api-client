/*
	Copyright 2023 Supercomputing Systems AG
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

//! Example that shows how to detect a runtime update and afterwards update the metadata.
use kitchensink_runtime::{RuntimeCall, UncheckedExtrinsic};
pub use pallet_balances::Call as BalancesCall;
use sp_core::{
	crypto::{AccountId32, Ss58Codec},
	sr25519, Bytes, Encode, Pair,
};
use sp_runtime::MultiAddress;
use std::process::ExitCode;
use substrate_api_client::ac_primitives::{AssetRuntimeConfig, ExtrinsicSigner};
fn main() -> Result<ExitCode, i32> {
	let alice: sr25519::Pair = Pair::from_string(
		"0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
		None,
	)
	.unwrap();

	let bob_account: AccountId32 =
		sr25519::Public::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")
			.unwrap()
			.into();
	let bob = MultiAddress::Id(bob_account);
	let es_converted: ExtrinsicSigner<AssetRuntimeConfig> = alice.clone().into();
	let es_new = ExtrinsicSigner::<AssetRuntimeConfig>::new(alice.clone());
	assert_eq!(es_converted.signer().public(), es_new.signer().public());

	let call1 = RuntimeCall::Balances(BalancesCall::force_transfer {
		source: bob.clone(),
		dest: bob.clone(),
		value: 10,
	});
	let call2 = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
		dest: bob.clone(),
		value: 2000,
	});
	let call3 =
		RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: bob, value: 1000 });

	let _xt1: Bytes = UncheckedExtrinsic::new_unsigned(call1).encode().into();
	let _xt2: Bytes = UncheckedExtrinsic::new_unsigned(call2).encode().into();
	let _xt3: Bytes = UncheckedExtrinsic::new_unsigned(call3).encode().into();
	/*
	let recipients_extrinsic_address: ExtrinsicAddressOf<AssetExtrinsicSigner> =
		bob_account.clone().into();

	//let recipient = AccountKeyring::Bob.to_account_id();
	//assert!(4 == 5, "x wasn't true!");
	let spec_version = 1;
	let transaction_version = 2;
	let genesis_hash = H256::zero();
	//let metadata = Metadata::new();
	let signer_nonce = 3;
	println!("[+] Alice's Account Nonce is {}", signer_nonce);

	// Construct an extrinsic using only functionality available in no_std
	let xt = {
		let extrinsic_params = <AssetRuntimeConfig as Config>::ExtrinsicParams::new(
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
	*/
	Ok(ExitCode::from(0))
}
