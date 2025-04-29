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

//! Old style Extrinsic Parameters for older Substrate chains that do not yet use the newer transaction Logic.
//!
// E.g in the runtime/src/lib.rs file:
// pub type TxExtension = (
// 	frame_system::CheckNonZeroSender<Runtime>,
// 	frame_system::CheckSpecVersion<Runtime>,
// 	frame_system::CheckTxVersion<Runtime>,
// 	frame_system::CheckGenesis<Runtime>,
// 	frame_system::CheckEra<Runtime>,
// 	frame_system::CheckNonce<Runtime>,
// 	frame_system::CheckWeight<Runtime>,
// 	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
// );

use crate::{
	config::Config,
	extrinsic_params::{ExtrinsicParams, GenericAdditionalParams, GenericTxExtension},
};
use codec::{Codec, Decode, DecodeWithMemTracking, Encode};
use scale_info::{StaticTypeInfo, TypeInfo};
use sp_runtime::{
	generic::Era,
	impl_tx_ext_default,
	traits::{Dispatchable, TransactionExtension},
};

#[derive(Decode, Encode, Copy, Clone, Eq, PartialEq, Debug, TypeInfo)]
pub struct TxExtensionWithoutHashCheck<Tip, Index> {
	pub era: Era,
	#[codec(compact)]
	pub nonce: Index,
	pub tip: Tip,
}

impl<Tip, Index> TxExtensionWithoutHashCheck<Tip, Index> {
	pub fn new(era: Era, nonce: Index, tip: Tip) -> Self {
		{
			Self { era, nonce, tip }
		}
	}
}

impl<Call, Tip, Index> TransactionExtension<Call> for TxExtensionWithoutHashCheck<Tip, Index>
where
	Call: Dispatchable,
	TxExtensionWithoutHashCheck<Tip, Index>: Codec
		+ core::fmt::Debug
		+ Sync
		+ Send
		+ Clone
		+ Eq
		+ PartialEq
		+ StaticTypeInfo
		+ DecodeWithMemTracking,
	Tip: Codec
		+ core::fmt::Debug
		+ Sync
		+ Send
		+ Clone
		+ Eq
		+ PartialEq
		+ StaticTypeInfo
		+ DecodeWithMemTracking,
	Index: Codec
		+ core::fmt::Debug
		+ Sync
		+ Send
		+ Clone
		+ Eq
		+ PartialEq
		+ StaticTypeInfo
		+ DecodeWithMemTracking,
{
	const IDENTIFIER: &'static str = "TxExtensionWithoutHashCheck";
	type Implicit = ();
	type Pre = ();
	type Val = ();

	impl_tx_ext_default!(Call; weight validate prepare);
}

pub type ImplicitWithoutHashCheck<Hash> = ((), u32, u32, Hash, Hash, (), (), ());

/// An implementation of [`ExtrinsicParams`] that is suitable for constructing
/// extrinsics that can be sent to a node with the same signed extra and additional
/// parameters as a Polkadot/Substrate node.
#[derive(Decode, Encode, Clone, Eq, PartialEq, Debug)]
pub struct ExtrinsicParamsWithoutHashCheck<T: Config, Tip> {
	era: Era,
	nonce: T::Index,
	tip: Tip,
	spec_version: u32,
	transaction_version: u32,
	genesis_hash: T::Hash,
	mortality_checkpoint: T::Hash,
}

impl<T, Tip> ExtrinsicParams<T::Index, T::Hash> for ExtrinsicParamsWithoutHashCheck<T, Tip>
where
	T: Config,
	u128: From<Tip>,
	Tip: Copy + Default + Encode,
{
	type AdditionalParams = GenericAdditionalParams<Tip, T::Hash>;
	type TxExtension = GenericTxExtension<Tip, T::Index>;
	type Implicit = ImplicitWithoutHashCheck<T::Hash>;

	fn new(
		spec_version: u32,
		transaction_version: u32,
		nonce: T::Index,
		genesis_hash: T::Hash,
		additional_params: Self::AdditionalParams,
	) -> Self {
		Self {
			era: additional_params.era,
			tip: additional_params.tip,
			spec_version,
			transaction_version,
			genesis_hash,
			mortality_checkpoint: additional_params.mortality_checkpoint.unwrap_or(genesis_hash),
			nonce,
		}
	}

	fn signed_extra(&self) -> Self::TxExtension {
		self.transaction_extension()
	}

	fn transaction_extension(&self) -> Self::TxExtension {
		Self::TxExtension::new(self.era, self.nonce, self.tip)
	}

	fn additional_signed(&self) -> Self::Implicit {
		self.implicit()
	}

	fn implicit(&self) -> Self::Implicit {
		{
			(
				(),
				self.spec_version,
				self.transaction_version,
				self.genesis_hash,
				self.mortality_checkpoint,
				(),
				(),
				(),
			)
		}
	}
}
