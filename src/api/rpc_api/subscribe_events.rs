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
	api::{error::Error, Api, Result},
	rpc::{HandleSubscription, Subscribe},
};
use ac_node_api::{events::EventDetails, Events, StaticEvent};
use ac_primitives::{ExtrinsicParams, FrameSystemConfig, StorageChangeSet};
use log::*;
use serde::de::DeserializeOwned;

// FIXME: This should rather be implemented directly on the
// Subscription return value, rather than the api. Or directly
// subcribe. Should be looked at in #288
// https://github.com/scs/substrate-api-client/issues/288#issuecomment-1346221653
pub trait SubscribeEvents<Client, Hash>
where
	Client: Subscribe,
	Hash: DeserializeOwned,
{
	fn wait_for_event<Ev: StaticEvent>(
		&self,
		subscription: &mut Client::Subscription<StorageChangeSet<Hash>>,
	) -> Result<Ev>;

	fn wait_for_event_details<Ev: StaticEvent>(
		&self,
		subscription: &mut Client::Subscription<StorageChangeSet<Hash>>,
	) -> Result<EventDetails>;
}

impl<Signer, Client, Params, Runtime> SubscribeEvents<Client, Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Client: Subscribe,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	fn wait_for_event<Ev: StaticEvent>(
		&self,
		subscription: &mut Client::Subscription<StorageChangeSet<Runtime::Hash>>,
	) -> Result<Ev> {
		let maybe_event_details = self.wait_for_event_details::<Ev>(subscription)?;
		maybe_event_details
			.as_event()?
			.ok_or(Error::Other("Could not find the specific event".into()))
	}

	fn wait_for_event_details<Ev: StaticEvent>(
		&self,
		subscription: &mut Client::Subscription<StorageChangeSet<Runtime::Hash>>,
	) -> Result<EventDetails> {
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
				event_details.check_if_failed()?;

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
