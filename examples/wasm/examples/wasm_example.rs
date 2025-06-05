/*
	Copyright 2024 Supercomputing Systems AG
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

//! Example that some basic functions that can be executed in WebAssembly.

pub use pallet_balances::Call as BalancesCall;
use sp_core::{
	crypto::{AccountId32, Ss58Codec},
	sr25519, Pair,
};
use sp_runtime::MultiAddress;
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
use std::process::ExitCode;
use substrate_api_client::ac_primitives::{ExtrinsicSigner, RococoRuntimeConfig};

fn main() -> Result<ExitCode, i32> {
	// This test is not yet very sophisticated and not exhaustive.
	// Still it shows how some basic data structures can be constructed and used.
	let alice: sr25519::Pair = Pair::from_string(
		"0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
		None,
	)
	.unwrap();

	let bob_account: AccountId32 =
		sr25519::Public::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")
			.unwrap()
			.into();
	let _bob: MultiAddress<AccountId32, AccountId32> = MultiAddress::Id(bob_account);
	let es_converted: ExtrinsicSigner<RococoRuntimeConfig> = alice.clone().into();
	let es_new = ExtrinsicSigner::<RococoRuntimeConfig>::new(alice.clone());
	assert_eq!(es_converted.signer().public(), es_new.signer().public());

	let extrinsic = UncheckedExtrinsic::from_bytes(&[]);
	if extrinsic.is_ok() {
		panic!("Extrinsic should be invalid")
	}
	Ok(ExitCode::from(0))
}
