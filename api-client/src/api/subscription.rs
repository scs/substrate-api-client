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

use crate::{
	api::{error::Error, Api, ApiResult, TransactionStatus},
	rpc::{HandleSubscription, Request, Subscribe},
	utils, FromHexString, XtStatus,
};
use ac_compose_macros::rpc_params;
pub use ac_node_api::{events::EventDetails, StaticEvent};
use ac_node_api::{DispatchError, Events};
use ac_primitives::{BalancesConfig, ExtrinsicParams};
use core::str::FromStr;
use log::*;
use serde::de::DeserializeOwned;
use sp_core::{storage::StorageChangeSet, Pair};
use sp_rpc::number::NumberOrHex;
use sp_runtime::MultiSigner;

pub type TransactionSubscriptionFor<Client, Hash> =
	<Client as Subscribe>::Subscription<TransactionStatus<Hash, Hash>>;

impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Signer: Pair,
	MultiSigner: From<Signer::Public>,
	Client: Subscribe + Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: BalancesConfig,
	Runtime::Hash: FromHexString,
	Runtime::Balance: TryFrom<NumberOrHex> + FromStr,
	Runtime::Header: DeserializeOwned,
{
	/// Submit an extrinsic an return a websocket Subscription to watch the
	/// extrinsic progress.
	pub fn submit_and_watch_extrinsic(
		&self,
		xthex_prefixed: &str,
	) -> ApiResult<TransactionSubscriptionFor<Client, Runtime::Hash>> {
		self.client()
			.subscribe(
				"author_submitAndWatchExtrinsic",
				rpc_params![xthex_prefixed],
				"author_unsubmitAndWatchExtrinsic",
			)
			.map_err(|e| e.into())
	}

	/// Submit an extrinsic and watch in until the desired status is reached,
	/// if no error is encountered previously. This method is blocking.
	pub fn submit_and_watch_extrinsic_until(
		&self,
		xthex_prefixed: &str,
		watch_until: XtStatus,
	) -> ApiResult<Option<Runtime::Hash>> {
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

	pub fn subscribe_events(
		&self,
	) -> ApiResult<Client::Subscription<StorageChangeSet<Runtime::Hash>>> {
		debug!("subscribing to events");
		let key = utils::storage_key("System", "Events");
		self.client()
			.subscribe("state_subscribeStorage", rpc_params![vec![key]], "state_unsubscribeStorage")
			.map_err(|e| e.into())
	}

	pub fn subscribe_finalized_heads(&self) -> ApiResult<Client::Subscription<Runtime::Header>> {
		debug!("subscribing to finalized heads");
		self.client()
			.subscribe(
				"chain_subscribeFinalizedHeads",
				rpc_params![],
				"chain_unsubscribeFinalizedHeads",
			)
			.map_err(|e| e.into())
	}

	pub fn wait_for_event<Ev: StaticEvent>(
		&self,
		subscription: &mut Client::Subscription<StorageChangeSet<Runtime::Hash>>,
	) -> ApiResult<Ev> {
		let maybe_event_details = self.wait_for_event_details::<Ev>(subscription)?;
		maybe_event_details
			.as_event()?
			.ok_or(Error::Other("Could not find the specific event".into()))
	}

	pub fn wait_for_event_details<Ev: StaticEvent>(
		&self,
		subscription: &mut Client::Subscription<StorageChangeSet<Runtime::Hash>>,
	) -> ApiResult<EventDetails> {
		while let Some(change_set) = subscription.next() {
			let event_bytes = change_set?.changes[0].1.as_ref().unwrap().0.clone();
			let events = Events::<Runtime::Hash>::new(
				self.metadata().clone(),
				Default::default(),
				event_bytes,
			);
			for maybe_event_details in events.iter() {
				let event_details = maybe_event_details?;

				// Check for failed xt and return as Dispatch Error in case we find one.
				// Careful - this reports the first one encountered. This event may belong to another extrinsic
				// than the one that is being waited for.
				if extrinsic_has_failed(&event_details) {
					let dispatch_error =
						DispatchError::decode_from(event_details.field_bytes(), self.metadata());
					return Err(Error::Dispatch(dispatch_error))
				}

				let event_metadata = event_details.event_metadata();
				trace!(
					"Found extrinsic: {:?}, {:?}",
					event_metadata.pallet(),
					event_metadata.event()
				);
				if event_metadata.pallet() == Ev::PALLET && event_metadata.event() == Ev::EVENT {
					return Ok(event_details)
				} else {
					trace!("Not the event we are looking for, skipping.")
				}
			}
		}
		Err(Error::NoStream)
	}
}

fn extrinsic_has_failed(event_details: &EventDetails) -> bool {
	event_details.pallet_name() == "System" && event_details.variant_name() == "ExtrinsicFailed"
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
