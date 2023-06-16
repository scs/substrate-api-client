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

use crate::LocalKeystore;
use sc_keystore::Result;
use sp_application_crypto::{AppPair, AppPublic};

/// This is an extension from the substrate-api-client repo. Keep it as a separate trait to
/// make that clear.
pub trait KeystoreExt {
	fn generate<Pair: AppPair>(&self) -> Result<Pair>;
	fn public_keys<Public: AppPublic>(&self) -> Result<Vec<Public>>;
}

impl KeystoreExt for LocalKeystore {
	fn generate<Pair: AppPair>(&self) -> Result<Pair> {
		self.0.write().generate_by_type::<Pair::Generic>(Pair::ID).map(Into::into)
	}

	fn public_keys<Public: AppPublic>(&self) -> Result<Vec<Public>> {
		self.0
			.read()
			.raw_public_keys(Public::ID)
			.map(|v| v.into_iter().filter_map(|k| Public::from_slice(k.as_slice()).ok()).collect())
	}
}

#[cfg(test)]
mod tests {
	use crate::{KeystoreExt, LocalKeystore};
	use sp_application_crypto::sr25519::AppPair;
	use sp_core::Pair;

	#[test]
	fn test_execute_generate_doesnt_fail() {
		let store = LocalKeystore::in_memory();
		let generated_key = store.generate::<AppPair>();

		// check that something was generated
		assert_ne!("", format!("{:?}", generated_key.unwrap().to_raw_vec()));
	}
}
