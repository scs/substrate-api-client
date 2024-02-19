/*
	Copyright 2023 Supercomputing Systems AG
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
use crate::{api::Error, rpc::Subscribe, rpc_api::EventSubscriptionFor, Result};
use alloc::sync::Arc;
use codec::{Decode, Encode};
use core::sync::atomic::{AtomicBool, Ordering};
use serde::de::DeserializeOwned;

/// Struct to support waiting for runtime updates.
pub struct RuntimeUpdateDetector<Hash, Client>
where
	Hash: DeserializeOwned + Copy + Decode,
	Client: Subscribe,
{
	subscription: EventSubscriptionFor<Client, Hash>,
	external_cancellation: Option<Arc<AtomicBool>>,
}

impl<Hash, Client> RuntimeUpdateDetector<Hash, Client>
where
	Hash: DeserializeOwned + Copy + Encode + Decode,
	Client: Subscribe,
{
	pub fn new(subscription: EventSubscriptionFor<Client, Hash>) -> Self {
		Self { subscription, external_cancellation: None }
	}

	/// Provide the `RuntimeUpdateDetector` with the additional option to cancel the waiting
	/// from the outside.
	pub fn new_with_cancellation(
		subscription: EventSubscriptionFor<Client, Hash>,
		cancellation: Arc<AtomicBool>,
	) -> Self {
		Self { subscription, external_cancellation: Some(cancellation) }
	}

	/// Returns true if a runtime update was detected, false if the wait was cancelled
	/// If not cancelled, this method only returns/resolves once a runtime update is detected.
	#[maybe_async::maybe_async(?Send)]
	pub async fn detect_runtime_update(&mut self) -> Result<bool> {
		'outer: loop {
			if let Some(canceled) = &self.external_cancellation {
				if canceled.load(Ordering::SeqCst) {
					return Ok(false)
				}
			}
			let event_records = self
				.subscription
				.next_events_from_metadata()
				.await
				.ok_or(Error::Other("Error receiving events".into()))??;
			let event_iter = event_records.iter();
			for event in event_iter {
				if event?.is_code_update() {
					break 'outer
				}
			}
		}
		Ok(true)
	}
}
