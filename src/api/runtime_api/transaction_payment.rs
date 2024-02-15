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

use super::{RuntimeApi, RuntimeApiClient};
use crate::{api::Result, rpc::Request};
use ac_primitives::{
	config::Config, FeeDetails, RuntimeDispatchInfo, UncheckedExtrinsicV4, Weight,
};
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use alloc::{vec, vec::Vec};
use sp_core::Encode;

#[maybe_async::maybe_async(?Send)]
pub trait TransactionPaymentApi: RuntimeApi {
	type FeeDetails;
	type RuntimeDispatchInfo;
	type Balance;
	type Weight;

	/// Query the transaction fee details.
	async fn query_fee_details<Address, Call, Signature, SignedExtra>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
		length: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Self::FeeDetails>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode;

	/// Query the transaction fee details of opaque extrinsic.
	async fn query_fee_details_opaque(
		&self,
		extrinsic: Vec<u8>,
		length: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Self::FeeDetails>;

	/// Query the transaction fee info.
	async fn query_info<Address, Call, Signature, SignedExtra>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
		length: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Self::RuntimeDispatchInfo>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode;

	/// Query the transaction info of opaque extrinsic.
	async fn query_info_opaque(
		&self,
		extrinsic: Vec<u8>,
		length: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Self::RuntimeDispatchInfo>;

	/// Query the output of the current LengthToFee given some input.
	async fn query_length_to_fee(
		&self,
		length: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Self::Balance>;

	/// Query the output of the current WeightToFee given some input.
	async fn query_weight_to_fee(
		&self,
		weight: Self::Weight,
		at_block: Option<Self::Hash>,
	) -> Result<Self::Balance>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> TransactionPaymentApi for RuntimeApiClient<T, Client>
where
	T: Config,
	Client: Request,
{
	type FeeDetails = FeeDetails<T::Balance>;
	type RuntimeDispatchInfo = RuntimeDispatchInfo<T::Balance>;
	type Balance = T::Balance;
	type Weight = Weight;

	async fn query_fee_details<Address, Call, Signature, SignedExtra>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
		length: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Self::FeeDetails>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode,
	{
		self.query_fee_details_opaque(extrinsic.encode(), length, at_block).await
	}

	async fn query_fee_details_opaque(
		&self,
		extrinsic: Vec<u8>,
		length: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Self::FeeDetails> {
		self.runtime_call(
			"TransactionPaymentApi_query_fee_details",
			vec![extrinsic, length.encode()],
			at_block,
		)
		.await
	}

	async fn query_info<Address, Call, Signature, SignedExtra>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
		length: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Self::RuntimeDispatchInfo>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode,
	{
		self.query_info_opaque(extrinsic.encode(), length, at_block).await
	}

	async fn query_info_opaque(
		&self,
		extrinsic: Vec<u8>,
		length: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Self::RuntimeDispatchInfo> {
		self.runtime_call(
			"TransactionPaymentApi_query_info",
			vec![extrinsic, length.encode()],
			at_block,
		)
		.await
	}

	async fn query_length_to_fee(
		&self,
		length: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Self::Balance> {
		self.runtime_call(
			"TransactionPaymentApi_query_length_to_fee",
			vec![length.encode()],
			at_block,
		)
		.await
	}

	async fn query_weight_to_fee(
		&self,
		weight: Self::Weight,
		at_block: Option<Self::Hash>,
	) -> Result<Self::Balance> {
		self.runtime_call(
			"TransactionPaymentApi_query_weight_to_fee",
			vec![weight.encode()],
			at_block,
		)
		.await
	}
}
