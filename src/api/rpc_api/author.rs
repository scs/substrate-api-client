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

//! Interface to common frame system pallet information.

use crate::{
	api::{Error, Result},
	rpc::{HandleSubscription, Request, Subscribe},
	utils::ToHexString,
	Api, ExtrinsicReport, FromHexString, TransactionStatus, XtStatus,
};
use ac_compose_macros::rpc_params;
use ac_primitives::{ExtrinsicParams, FrameSystemConfig};
use alloc::{format, vec::Vec};
use log::*;
use serde::de::DeserializeOwned;
use sp_runtime::traits::Hash as HashTrait;

pub type TransactionSubscriptionFor<Client, Hash> =
	<Client as Subscribe>::Subscription<TransactionStatus<Hash, Hash>>;

/// Simple extrinsic submission without any subscription.
pub trait SubmitExtrinsic {
	type Hash;

	/// Submit an extrsinic to the substrate node, without watching.
	/// Retruns the extrinsic hash.
	fn submit_extrinsic(&self, encoded_extrinsic: Vec<u8>) -> Result<Self::Hash>;
}

impl<Signer, Client, Params, Runtime> SubmitExtrinsic for Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	type Hash = Runtime::Hash;

	fn submit_extrinsic(&self, encoded_extrinsic: Vec<u8>) -> Result<Self::Hash> {
		let hex_encoded_xt = encoded_extrinsic.to_hex();
		debug!("sending extrinsic: {:?}", hex_encoded_xt);
		let xt_hash =
			self.client().request("author_submitExtrinsic", rpc_params![hex_encoded_xt])?;
		Ok(xt_hash)
	}
}

pub trait SubmitAndWatch<Client, Hash>
where
	Client: Subscribe,
	Hash: DeserializeOwned,
{
	/// Submit an extrinsic an return a websocket Subscription to watch the
	/// extrinsic progress.
	fn submit_and_watch_extrinsic(
		&self,
		encoded_extrinsic: Vec<u8>,
	) -> Result<TransactionSubscriptionFor<Client, Hash>>;

	/// Submit an extrinsic and watch in until the desired status is reached,
	/// if no error is encountered previously.
	// This method is blocking.
	fn submit_and_watch_extrinsic_until(
		&self,
		encoded_extrinsic: Vec<u8>,
		watch_until: XtStatus,
	) -> Result<ExtrinsicReport<Hash>>;
}

impl<Signer, Client, Params, Runtime> SubmitAndWatch<Client, Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Client: Subscribe,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
	Runtime::Hashing: HashTrait<Output = Runtime::Hash>,
	Runtime::Hash: FromHexString,
{
	fn submit_and_watch_extrinsic(
		&self,
		encoded_extrinsic: Vec<u8>,
	) -> Result<TransactionSubscriptionFor<Client, Runtime::Hash>> {
		self.client()
			.subscribe(
				"author_submitAndWatchExtrinsic",
				rpc_params![encoded_extrinsic.to_hex()],
				"author_unsubmitAndWatchExtrinsic",
			)
			.map_err(|e| e.into())
	}

	fn submit_and_watch_extrinsic_until(
		&self,
		encoded_extrinsic: Vec<u8>,
		watch_until: XtStatus,
	) -> Result<ExtrinsicReport<Runtime::Hash>> {
		let tx_hash = Runtime::Hashing::hash_of(&encoded_extrinsic);
		let mut subscription: TransactionSubscriptionFor<Client, Runtime::Hash> =
			self.submit_and_watch_extrinsic(encoded_extrinsic)?;

		while let Some(transaction_status) = subscription.next() {
			let transaction_status = transaction_status?;
			if transaction_status.is_supported() {
				if transaction_status.reached_status(watch_until) {
					subscription.unsubscribe()?;
					let block_hash = get_maybe_block_hash(transaction_status.clone());
					return Ok(ExtrinsicReport::new(tx_hash, block_hash, transaction_status, None))
				}
			} else {
				subscription.unsubscribe()?;
				let error = Error::Extrinsic(format!(
					"Unsupported transaction status: {:?}, stopping watch process.",
					transaction_status
				));
				return Err(error)
			}
		}
		Err(Error::NoStream)
	}
}

fn get_maybe_block_hash<Hash, BlockHash>(
	transcation_status: TransactionStatus<Hash, BlockHash>,
) -> Option<BlockHash> {
	match transcation_status {
		TransactionStatus::InBlock(block_hash) => Some(block_hash),
		TransactionStatus::Retracted(block_hash) => Some(block_hash),
		TransactionStatus::FinalityTimeout(block_hash) => Some(block_hash),
		TransactionStatus::Finalized(block_hash) => Some(block_hash),
		_ => None,
	}
}
