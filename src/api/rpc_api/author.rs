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
	TransactionReport, GetBlock,
	TransactionStatus, XtStatus, Events, GetStorage, Phase,
};
use ac_compose_macros::rpc_params;
use ac_primitives::{ExtrinsicParams, FrameSystemConfig};
use log::*;
use serde::de::DeserializeOwned;
use sp_runtime::traits::Hash as HashTrait;
use sp_runtime::traits::Block as BlockTrait;
use sp_runtime::traits::GetRuntimeBlockType;
use codec::Encode;
use crate::utils;
use crate::FromHexString;

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
	/// if no error is encountered previously.
	// This method is blocking.
	fn submit_and_watch_extrinsic_until(
		&self,
		xthex_prefixed: &str,
		watch_until: XtStatus,
	) -> Result<TransactionReport<Hash>>;

	/// Submit an extrinsic and watch in until
	/// - wait_for_finalized = false => InBlock
	/// - wait_for_finalized = false => Finalized
	/// and check if the extrinsic has been successful or not.
	// This method is blocking.
	fn submit_and_watch_extrinsic_until_success(
		&self,
		xthex_prefixed: &str,
		wait_for_finalized: bool,
	) -> Result<Option<Hash>>;
}

impl<Signer, Client, Params, Runtime> SubmitAndWatch<Client, Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Client: Subscribe + Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig + GetRuntimeBlockType,
	Runtime::RuntimeBlock: BlockTrait + DeserializeOwned,
	Runtime::Hashing: HashTrait<Output = Runtime::Hash>,
	Runtime::Hash: FromHexString,
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
		) -> Result<TransactionReport<Runtime::Hash>> {
		let tx_hash = Runtime::Hashing::hash_of(&xthex_prefixed.encode());
		let mut subscription: TransactionSubscriptionFor<Client, Runtime::Hash> =
			self.submit_and_watch_extrinsic(xthex_prefixed)?;

		while let Some(transaction_status) = subscription.next() {
			let transaction_status = transaction_status?;
			if transaction_status.is_supported() {
				if transaction_status.as_u8() >= watch_until as u8 {
					subscription.unsubscribe()?;
					let block_hash = get_maybe_block_hash(transaction_status);
					return Ok(TransactionReport::new(tx_hash, block_hash, transaction_status, None))
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


		/// Submit an extrinsic and watch in until inBlock or Finalized is reached,
	/// if no error is encountered previously. This method is blocking.
	fn submit_and_watch_extrinsic_until_success(
		&self,
		xthex_prefixed: &str,
		wait_for_finalized: bool,
	) -> Result<Option<Runtime::Hash>> {
		let xt_status = match wait_for_finalized {
			true => XtStatus::Finalized,
			false => XtStatus::InBlock,
		};
		let mut report = self.submit_and_watch_extrinsic_until(xthex_prefixed, xt_status)?;

		// Retrieve block details from node.
		let block_hash = report.block_hash.ok_or(Error::NoBlockHash)?;
		let block = self.get_block(Some(block_hash))?.ok_or(Error::NoBlock)?;
		let xt_index = block.extrinsics().iter().position(|xt|
			let xt_hash = Runtime::Hashing::hash_of(&xt.encode())
			report.xt_hash == xt_hash
		).ok_or(Error::Extrinsic("Could not find extrinsic hash".to_string()))?;

		// Fetch events from this block.
		let key = utils::storage_key("System", "Events");
		let event_bytes = self.get_opaque_storage_by_key_hash(key, Some(block_hash))?.ok_or(Error::NoBlock)?;
		let events = Events::<Runtime::Hash>::new(
			self.metadata().clone(),
			Default::default(),
			event_bytes,
		);

		// Filter events associated to our extrinsic.
		let associated_events = events.iter().filter(|ev | { ev.map(ev.phase() == Phase::ApplyExtrinsic(xt_index as u32))?})?;

		for event in associated_events.iter() {
			if extrinsic_has_failed(&event_details) {
				let dispatch_error =
					DispatchError::decode_from(event_details.field_bytes(), self.metadata());
				return Err(Error::Dispatch(dispatch_error))
			}
		}

		report.events = Some(associated_events)
		Ok(report)


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
