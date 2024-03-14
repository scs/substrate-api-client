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

//! Tests for the author rpc interface functions.

use ac_keystore::{Keystore, KeystoreExt, LocalKeystore};
use sp_application_crypto::sr25519;
use sp_core::crypto::{KeyTypeId, Ss58Codec};
use std::path::PathBuf;

pub const KEYSTORE_PATH: &str = "my_keystore";
pub const SR25519: KeyTypeId = KeyTypeId(*b"sr25");

fn main() {
	let store = LocalKeystore::open(PathBuf::from(&KEYSTORE_PATH), None).unwrap();
	let seed = "//Ferdie";

	// This does not place the key into the keystore if we have a seed, but it does
	// place it into the keystore if the seed is none.
	let key = store.sr25519_generate_new(SR25519, Some(seed)).unwrap();
	store.insert(SR25519, seed, &key.0).unwrap();

	drop(store);
	println!("{}", key.to_ss58check());

	let store = LocalKeystore::open(PathBuf::from(&KEYSTORE_PATH), None).unwrap();
	let pubkeys = store.public_keys::<sr25519::AppPublic>().unwrap();

	assert_eq!(pubkeys[0].to_ss58check(), key.to_ss58check());
}
