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
	rpc::{Error as RpcClientError, HandleSubscription, Subscribe},
	utils, Hash, Index, XtStatus,
};
use ac_compose_macros::rpc_params;
pub use ac_node_api::{events::EventDetails, StaticEvent};
use ac_node_api::{DispatchError, Events};
use ac_primitives::ExtrinsicParams;
use core::fmt::Debug;
use log::*;
use sp_core::{storage::StorageChangeSet, Pair, H256};
use sp_runtime::{DeserializeOwned, MultiSigner};

impl<P, Client, Params> Api<P, Client, Params>
where
	P: Pair,
	MultiSigner: From<P::Public>,
	Client: Subscribe,
	Params: ExtrinsicParams<Index, Hash>,
{
	pub fn watch_extrinsic<Hash: DeserializeOwned, BlockHash: DeserializeOwned>(
		&self,
		xthex_prefixed: &str,
	) -> ApiResult<Client::Subscription<TransactionStatus<Hash, BlockHash>>> {
		self.client()
			.subscribe(
				"author_submitAndWatchExtrinsic",
				rpc_params![xthex_prefixed],
				"author_unsubmitAndWatchExtrinsic",
			)
			.map_err(|e| e.into())
	}

	pub fn watch_extrinsic_until<
		Hash: DeserializeOwned + Debug,
		BlockHash: DeserializeOwned + Debug,
	>(
		&self,
		xthex_prefixed: &str,
		watch_until: XtStatus,
	) -> ApiResult<Option<BlockHash>> {
		let mut subscription: Client::Subscription<TransactionStatus<Hash, BlockHash>> =
			self.watch_extrinsic(xthex_prefixed)?;
		while let Some(transaction_status) = subscription.next() {
			let transaction_status = transaction_status?;
			if transaction_status.is_supported() {
				if transaction_status.as_u8() >= watch_until as u8 {
					return Ok(return_block_hash_if_available(transaction_status))
				}
			} else {
				let error = RpcClientError::Extrinsic(format!(
					"Unsupported transaction status: {:?}, stopping watch process.",
					transaction_status
				));
				return Err(error.into())
			}
		}
		Err(Error::RpcClient(RpcClientError::NoStream))
	}

	pub fn subscribe_events(&self) -> ApiResult<Client::Subscription<StorageChangeSet<H256>>> {
		debug!("subscribing to events");
		let key = utils::storage_key("System", "Events");
		self.client()
			.subscribe("state_subscribeStorage", rpc_params![vec![key]], "state_unsubscribeStorage")
			.map_err(|e| e.into())
	}

	pub fn subscribe_finalized_heads<Header: DeserializeOwned>(
		&self,
	) -> ApiResult<Client::Subscription<Header>> {
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
		subscription: &mut Client::Subscription<StorageChangeSet<H256>>,
	) -> ApiResult<Ev> {
		let maybe_event_details = self.wait_for_event_details::<Ev>(subscription)?;
		maybe_event_details
			.as_event()?
			.ok_or(Error::Other("Could not find the specific event".into()))
	}

	pub fn wait_for_event_details<Ev: StaticEvent>(
		&self,
		subscription: &mut Client::Subscription<StorageChangeSet<H256>>,
	) -> ApiResult<EventDetails> {
		while let Some(change_set) = subscription.next() {
			let event_bytes = change_set?.changes[0].1.as_ref().unwrap().0.clone();
			let events = Events::new(self.metadata().clone(), Default::default(), event_bytes);
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
		Err(Error::RpcClient(RpcClientError::NoStream))
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
