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

use super::extensions::{CheckGenesis, VerifySignature};
use crate::config::Config;
use codec::{Codec, Decode, Encode};
use primitive_types::H256;
use scale_info::{StaticTypeInfo, TypeInfo};
use sp_core::crypto::AccountId32;
use sp_runtime::{
	generic::Era,
	impl_tx_ext_default,
	traits::{BlakeTwo256, Dispatchable, Hash, TransactionExtension},
	MultiSignature,
};

pub type TestTxExtension = (VerifySignature<MultiSignature, AccountId32>, CheckGenesis<H256>);

// #[derive(Decode, Encode, Copy, Clone, Eq, PartialEq, Debug, TypeInfo)]
// pub struct TestTxExtension<TxExtension> {
// 	pub inner: TxExtension,
// }
//
// impl<TxExtension> TestTxExtension<TxExtension> {
// 	pub fn new(inner: TxExtension) -> Self {
// 		{
// 			Self { inner }
// 		}
// 	}
// }

// impl<Call, Tip, Index> TransactionExtension<Call> for TestTxExtension<Tip, Index>
// where
// 	Call: Dispatchable,
// 	TestTxExtension<Tip, Index>:
// 		Codec + core::fmt::Debug + Sync + Send + Clone + Eq + PartialEq + StaticTypeInfo,
// 	Tip: Codec + core::fmt::Debug + Sync + Send + Clone + Eq + PartialEq + StaticTypeInfo,
// 	Index: Codec + core::fmt::Debug + Sync + Send + Clone + Eq + PartialEq + StaticTypeInfo,
// {
// 	const IDENTIFIER: &'static str = "TestTxExtension";
// 	type Implicit = ();
// 	type Pre = ();
// 	type Val = ();
//
// 	impl_tx_ext_default!(Call; weight validate prepare);
// }

#[cfg(test)]
mod tests {
	use super::*;
	use sp_keyring::Sr25519Keyring;
	use sp_runtime::MultiSignature;

	#[test]
	fn unsigned_codec_should_work() {
		let my_extension: TestTxExtension =
			(VerifySignature::new_disabled(), CheckGenesis::new(H256::default()));

		let implicit = my_extension.implicit();

		assert_eq!(implicit, ((), H256::default()));
	}
}
