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
	Api, TransactionStatus, XtStatus,
};
use ac_compose_macros::rpc_params;
use ac_primitives::{ExtrinsicParams, FrameSystemConfig};
use log::*;
use serde::de::DeserializeOwned;

pub type TransactionSubscriptionFor<Client, Hash> =
	<Client as Subscribe>::Subscription<TransactionStatus<Hash, Hash>>;

/// Simple extrinsic submission without any subscription.
pub trait SubmitExtrinsic {
	type Hash;

	/// Submit an extrsinic to the substrate node, without watching.
	/// Retruns the extrinsic hash.
	fn submit_extrinsic(&self, xthex_prefixed: String) -> Result<Self::Hash>;
}

impl<Signer, Client, Params, Runtime> SubmitExtrinsic for Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	type Hash = Runtime::Hash;

	fn submit_extrinsic(&self, xthex_prefixed: String) -> Result<Self::Hash> {
		debug!("sending extrinsic: {:?}", xthex_prefixed);
		let xt_hash =
			self.client().request("author_submitExtrinsic", rpc_params![xthex_prefixed])?;
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
		xthex_prefixed: &str,
	) -> Result<TransactionSubscriptionFor<Client, Hash>>;

	/// Submit an extrinsic and watch in until the desired status is reached,
	/// if no error is encountered previously. This method is blocking.
	fn submit_and_watch_extrinsic_until(
		&self,
		xthex_prefixed: &str,
		watch_until: XtStatus,
	) -> Result<Option<Hash>>;
}

impl<Signer, Client, Params, Runtime> SubmitAndWatch<Client, Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Client: Subscribe,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	fn submit_and_watch_extrinsic(
		&self,
		xthex_prefixed: &str,
	) -> Result<TransactionSubscriptionFor<Client, Runtime::Hash>> {
		self.client()
			.subscribe(
				"author_submitAndWatchExtrinsic",
				rpc_params![xthex_prefixed],
				"author_unsubmitAndWatchExtrinsic",
			)
			.map_err(|e| e.into())
	}

	fn submit_and_watch_extrinsic_until(
		&self,
		xthex_prefixed: &str,
		watch_until: XtStatus,
	) -> Result<Option<Runtime::Hash>> {
		let mut subscription: TransactionSubscriptionFor<Client, Runtime::Hash> =
			self.submit_and_watch_extrinsic(xthex_prefixed)?;
		while let Some(transaction_status) = subscription.next() {
			let transaction_status = transaction_status?;
			if transaction_status.is_supported() {
				if transaction_status.as_u8() >= watch_until as u8 {
					subscription.unsubscribe()?;
					return Ok(return_block_hash_if_available(transaction_status))
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

fn return_block_hash_if_available<Hash, BlockHash>(
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
