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

use crate::config::Config;
use codec::{Codec, Decode, Encode};
use primitive_types::H256;
use scale_info::{StaticTypeInfo, TypeInfo};
use sp_runtime::{
	generic::Era,
	impl_tx_ext_default,
	traits::{BlakeTwo256, Dispatchable, Hash, TransactionExtension},
};

pub trait ExtrinsicExtension {
	type Implicit;
	type TxExtension;

	fn implicit(&self) -> &Self::Implicit;
	fn tx_extension(&self) -> &Self::TxExtension;
}

/// Extension that, if enabled, validates a signature type against the payload constructed from the
/// call and the rest of the transaction extension pipeline. This extension provides the
/// functionality that traditionally signed transactions had with the implicit signature checking
/// implemented in [`Checkable`](sp_runtime::traits::Checkable). It is meant to be placed ahead of
/// any other extensions that do authorization work in the [`TransactionExtension`] pipeline.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub enum VerifySignature<Signature, AccountId> {
	/// The extension will verify the signature and, if successful, authorize a traditionally
	/// signed transaction.
	Signed {
		/// The signature provided by the transaction submitter.
		signature: Signature,
		/// The account that signed the payload.
		account: AccountId,
	},
	/// The extension is disabled and will be passthrough.
	Disabled,
}

impl<Signature, AccountId> VerifySignature<Signature, AccountId> {
	/// Create a new extension instance that will validate the provided signature.
	pub fn new_with_signature(signature: Signature, account: AccountId) -> Self {
		Self::Signed { signature, account }
	}

	/// Create a new passthrough extension instance.
	pub fn new_disabled() -> Self {
		Self::Disabled
	}
}
impl<Signature, AccountId> ExtrinsicExtension for VerifySignature<Signature, AccountId> {
	type Implicit = ();
	type TxExtension = Self;

	fn implicit(&self) -> &Self::Implicit {
		return &()
	}
	fn tx_extension(&self) -> &Self::TxExtension {
		return self
	}
}

/// Genesis hash check to provide replay protection between different networks.
///
/// # Transaction Validity
///
/// Note that while a transaction with invalid `genesis_hash` will fail to be decoded,
/// the extension does not affect any other fields of `TransactionValidity` directly.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckGenesis<Hash>(Hash);

impl<Hash> CheckGenesis<Hash> {
	pub fn new(hash: Hash) -> Self {
		CheckGenesis(hash)
	}
}

impl<Hash> ExtrinsicExtension for CheckGenesis<Hash> {
	type Implicit = Hash;
	type TxExtension = ();

	fn implicit(&self) -> &Self::Implicit {
		&self.0
	}
	fn tx_extension(&self) -> &Self::TxExtension {
		return &()
	}
}
