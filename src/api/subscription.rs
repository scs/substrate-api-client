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
	rpc::{HandleSubscription, Subscribe},
	utils, Hash, Index,
};
pub use ac_node_api::{events::EventDetails, StaticEvent};
use ac_node_api::{DispatchError, Events};
use ac_primitives::ExtrinsicParams;
use log::*;
use sp_core::Pair;
use sp_runtime::{MultiSignature, MultiSigner};
use std::sync::mpsc::{Receiver, Sender as ThreadOut};

impl<P, Client, Params> Api<P, Client, Params>
where
	P: Pair,
	MultiSigner: From<P::Public>,
	Client: Subscribe,
	Params: ExtrinsicParams<Index, Hash>,
{
	pub fn watch_extrinsic<Hash, BlockHash>(
		&self,
		xthex_prefixed: &str,
	) -> ApiResult<dyn HandleSubscription<TransactionStatus<Hash, BlockHash>>> {
		self.client
			.subscribe(
				"author_submitAndWatchExtrinsic",
				Some(xthex_prefixed),
				"author_unsubmitAndWatchExtrinsic",
			)
			.map_err(|e| e.into())
	}

	pub fn subscribe_events(&self) -> ApiResult<dyn HandleSubscription<Vec<u8>>> {
		debug!("subscribing to events");
		let key = utils::storage_key("System", "Events");
		self.client()
			.subscribe("state_subscribeStorage", Some(vec![key]), "state_unsubscribeStorage")
			.map_err(|e| e.into())
	}

	pub fn subscribe_finalized_heads<Header>(&self) -> ApiResult<dyn HandleSubscription<Header>> {
		debug!("subscribing to finalized heads");
		self.client()
			.subscribe("chain_subscribeFinalizedHeads", None, "chain_unsubscribeFinalizedHeads")
			.map_err(|e| e.into())
	}

	pub fn wait_for_event<Ev: StaticEvent>(
		&self,
		subscription: &dyn HandleSubscription<Vec<u8>>,
	) -> ApiResult<Ev> {
		let maybe_event_details = self.wait_for_event_details::<Ev>(subscription)?;
		maybe_event_details
			.as_event()?
			.ok_or(Error::Other("Could not find the specific event".into()))
	}

	pub fn wait_for_event_details<Ev: StaticEvent>(
		&self,
		subscription: &dyn HandleSubscription<Vec<u8>>,
	) -> ApiResult<EventDetails> {
		while let Some(event_bytes) = subscription.next() {
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
	}
}

fn extrinsic_has_failed(event_details: &EventDetails) -> bool {
	event_details.pallet_name() == "System" && event_details.variant_name() == "ExtrinsicFailed"
}
