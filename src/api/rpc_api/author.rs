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
	api::{rpc_api::events::FetchEvents, Error, Result},
	rpc::{HandleSubscription, Request, Subscribe},
	Api, ExtrinsicReport, FromHexString, ToHexString, TransactionStatus, XtStatus,
};
use ac_compose_macros::rpc_params;
use ac_primitives::{ExtrinsicParams, FrameSystemConfig};
use alloc::{format, vec::Vec};
use log::*;
use serde::de::DeserializeOwned;
use sp_runtime::traits::{Block as BlockTrait, GetRuntimeBlockType, Hash as HashTrait};

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

pub trait SubmitAndWatchUntilSuccess<Client, Hash>
where
	Client: Subscribe,
	Hash: DeserializeOwned,
{
	/// Submit an extrinsic and watch it until
	/// - wait_for_finalized = false => InBlock
	/// - wait_for_finalized = true => Finalized
	/// and check if the extrinsic has been successful or not.
	// This method is blocking.
	fn submit_and_watch_extrinsic_until_success(
		&self,
		encoded_extrinsic: Vec<u8>,
		wait_for_finalized: bool,
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
					let block_hash = transaction_status.get_maybe_block_hash();
					return Ok(ExtrinsicReport::new(
						tx_hash,
						block_hash.copied(),
						transaction_status,
						None,
					))
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

impl<Signer, Client, Params, Runtime> SubmitAndWatchUntilSuccess<Client, Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Client: Subscribe + Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime::Hashing: HashTrait<Output = Runtime::Hash>,
	Runtime: FrameSystemConfig + GetRuntimeBlockType,
	Runtime::RuntimeBlock: BlockTrait + DeserializeOwned,
	Runtime::Hashing: HashTrait<Output = Runtime::Hash>,
	Runtime::Hash: FromHexString,
{
	fn submit_and_watch_extrinsic_until_success(
		&self,
		encoded_extrinsic: Vec<u8>,
		wait_for_finalized: bool,
	) -> Result<ExtrinsicReport<Runtime::Hash>> {
		let xt_status = match wait_for_finalized {
			true => XtStatus::Finalized,
			false => XtStatus::InBlock,
		};
		let mut report = self.submit_and_watch_extrinsic_until(encoded_extrinsic, xt_status)?;

		let block_hash = report.block_hash.ok_or(Error::NoBlockHash)?;

		let extrinsic_events =
			self.fetch_events_for_extrinsic(block_hash, report.extrinsic_hash)?;

		// Ensure that the extrins has been successful. If not, return an error.
		for event in extrinsic_events.clone() {
			event.check_if_failed()?;
		}

		report.events = Some(extrinsic_events);
		Ok(report)
	}
}
