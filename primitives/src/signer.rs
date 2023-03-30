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

//! Signer used to sign extrinsic.

use crate::FrameSystemConfig;
use codec::{Decode, Encode};
use core::marker::PhantomData;
use sp_core::{crypto::AccountId32, Pair};
use sp_runtime::{traits::StaticLookup, MultiAddress};

pub trait SignExtrinsic<AccountId: Clone + Encode> {
	type Signature: Encode;
	type ExtrinsicAddress: Clone + Encode;

	/// Sign a given payload and return the resulting Signature.
	fn sign(&self, payload: &[u8]) -> Self::Signature;

	/// Return the public account id of the key pair.
	fn public_account_id(&self) -> &AccountId;

	/// Return the public address of the key pair. This is needed for the
	/// extrinsic creation, as substrate requires a Lookup transformation
	/// from Address to AccoundId.
	fn extrinsic_address(&self) -> Self::ExtrinsicAddress;
}

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct ExtrinsicSigner<Signer, Signature, Runtime>
where
	Signer: Pair,
	Runtime: FrameSystemConfig,
{
	signer: Signer,
	account_id: Runtime::AccountId,
	extrinsic_address: <Runtime::Lookup as StaticLookup>::Source,
	_phantom: PhantomData<Signature>,
}

impl<Signer, Signature, Runtime> ExtrinsicSigner<Signer, Signature, Runtime>
where
	Signer: Pair,
	Runtime: FrameSystemConfig,
	Runtime::AccountId: From<Signer::Public>,
{
	pub fn new(signer: Signer) -> Self {
		let account_id: Runtime::AccountId = signer.public().into();
		let extrinsic_address = Runtime::Lookup::unlookup(account_id.clone());
		Self { signer, account_id, extrinsic_address, _phantom: Default::default() }
	}

	pub fn signer(&self) -> &Signer {
		&self.signer
	}
}

impl<Signer, Signature, Runtime> SignExtrinsic<Runtime::AccountId>
	for ExtrinsicSigner<Signer, Signature, Runtime>
where
	Runtime: FrameSystemConfig,
	Signer: Pair,
	Signature: From<Signer::Signature> + Encode,
{
	type Signature = Signature;
	type ExtrinsicAddress = <Runtime::Lookup as StaticLookup>::Source;

	fn sign(&self, payload: &[u8]) -> Self::Signature {
		self.signer.sign(payload).into()
	}

	fn public_account_id(&self) -> &Runtime::AccountId {
		&self.account_id
	}

	fn extrinsic_address(&self) -> Self::ExtrinsicAddress {
		self.extrinsic_address.clone()
	}
}

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct StaticExtrinsicSigner<Signer, Signature> {
	signer: Signer,
	account_id: AccountId32,
	extrinsic_address: MultiAddress<AccountId32, u32>,
	_phantom: PhantomData<Signature>,
}

impl<Signer, Signature> StaticExtrinsicSigner<Signer, Signature>
where
	Signer: Pair,
	Signer::Public: Into<AccountId32>,
	Signature: From<Signer::Signature> + Encode + Clone,
{
	pub fn new(signer: Signer) -> Self {
		let account_id = signer.public().into();
		let extrinsic_address = MultiAddress::from(account_id.clone());
		Self { signer, account_id, extrinsic_address, _phantom: Default::default() }
	}

	pub fn signer(&self) -> &Signer {
		&self.signer
	}
}

impl<Signer, Signature> SignExtrinsic<AccountId32> for StaticExtrinsicSigner<Signer, Signature>
where
	Signer: Pair,
	Signature: From<Signer::Signature> + Encode + Clone,
{
	type Signature = Signature;
	type ExtrinsicAddress = MultiAddress<AccountId32, u32>;

	fn sign(&self, payload: &[u8]) -> Self::Signature {
		self.signer.sign(payload).into()
	}

	fn public_account_id(&self) -> &AccountId32 {
		&self.account_id
	}

	fn extrinsic_address(&self) -> Self::ExtrinsicAddress {
		self.extrinsic_address.clone()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use node_template_runtime::Signature;
	use sp_keyring::AccountKeyring;
	//use node_template_runtime::Runtime;

	/// This test fails!
	// 	#[test]
	// 	fn test_extrinsic_signer_clone() {
	// 		let pair = AccountKeyring::Alice.pair();
	// 		let signer = ExtrinsicSigner::<_, Signature, Runtime>::new(pair);
	//
	// 		let signer2 = signer.clone();
	// 	}

	#[test]
	fn test_simple_extrinsic_signer_clone() {
		let pair = AccountKeyring::Alice.pair();
		let signer = StaticExtrinsicSigner::<_, Signature>::new(pair);

		let _signer2 = signer.clone();
	}
}
