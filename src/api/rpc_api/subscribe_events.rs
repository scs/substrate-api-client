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
	utils,
};
use ac_compose_macros::rpc_params;
use ac_node_api::{Events, StaticEvent};
use ac_primitives::{ExtrinsicParams, FrameSystemConfig, StorageChangeSet};
use log::*;
use serde::de::DeserializeOwned;
use std::{marker::Sync, sync::mpsc::Sender};

pub trait SubscribeEvents<Client, Hash>
where
	Client: Subscribe,
	Hash: DeserializeOwned,
{
	/// Listens for a specific event type and only returns if an undecodeable
	/// Event is received or the event has been found.
	fn subscribe_for_event_type<Ev: StaticEvent + Sync + Send + 'static>(
		&self,
		sender: Sender<Ev>,
	) -> Result<()>;
}

impl<Signer, Client, Params, Runtime> SubscribeEvents<Client, Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Client: Subscribe,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	fn subscribe_for_event_type<Ev: StaticEvent + Sync + Send + 'static>(
		&self,
		sender: Sender<Ev>,
	) -> Result<()> {
		let subscription_key = utils::storage_key(Ev::PALLET, "Events");
		let mut subscription: Client::Subscription<StorageChangeSet<Runtime::Hash>> =
			self.client().subscribe(
				"state_subscribeStorage",
				rpc_params![vec![subscription_key]],
				"state_unsubscribeStorage",
			)?;

		while let Some(Ok(change_set)) = subscription.next() {
			// We only subscribed to one key, so always take the first value of the change set.
			if let Some(storage_data) = &change_set.changes[0].1 {
				let events = Events::<Runtime::Hash>::new(
					self.metadata().clone(),
					Default::default(),
					storage_data.0.clone(),
				);
				for event_details in events.iter().flatten() {
					match event_details.as_event::<Ev>() {
						Ok(Some(event)) => {
							sender.send(event).map_err(|e| Error::Other(Box::new(e)))?;
						},
						Ok(None) => {
							trace!(
								"Found extrinsic: {:?}, {:?}",
								event_details.event_metadata().pallet(),
								event_details.event_metadata().event()
							);
							trace!("Not the event we are looking for, skipping.");
						},
						Err(_) => error!("Could not decode event details."),
					}
				}
			}
		}

		Ok(())
	}
}
