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
use ac_node_api::{Events, StaticEvent};
use ac_primitives::{ExtrinsicParams, FrameSystemConfig, StorageChangeSet};
use log::*;
use serde::de::DeserializeOwned;
pub trait SubscribeEvents<Client, Hash>
where
	Client: Subscribe,
	Hash: DeserializeOwned,
{
	/// Listens for a specific event type and only returns if an undecodeable
	/// Event is received or the event has been found.
	fn wait_for_event<Ev: StaticEvent>(
		&self,
		subscription: &mut Client::Subscription<StorageChangeSet<Hash>>,
	) -> Result<Ev>;
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
		while let Some(change_set) = subscription.next() {
			let event_bytes = change_set?.changes[0].1.as_ref().unwrap().0.clone();
			let events = Events::<Runtime::Hash>::new(
				self.metadata().clone(),
				Default::default(),
				event_bytes,
			);
			for maybe_event_details in events.iter() {
				let event_details = maybe_event_details?;

				match event_details.as_event::<Ev>()? {
					Some(event) => return Ok(event),
					None => {
						trace!(
							"Found extrinsic: {:?}, {:?}",
							event_details.event_metadata().pallet(),
							event_details.event_metadata().event()
						);
						trace!("Not the event we are looking for, skipping.");
					},
				}
			}
		}
		Err(Error::NoStream)
	}
}
