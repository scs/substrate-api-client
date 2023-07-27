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
use crate::{ac_primitives::Config, rpc::Subscribe, rpc_api::EventSubscriptionFor};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct RuntimeUpdateDetector<T, Client>
where
	T: Config,
	Client: Subscribe,
{
	subscription: EventSubscriptionFor<Client, T::Hash>,
	external_cancellation: Option<Arc<AtomicBool>>,
}

impl<T, Client> RuntimeUpdateDetector<T, Client>
where
	T: Config,
	Client: Subscribe,
{
	pub fn new(subscription: EventSubscriptionFor<Client, T::Hash>) -> Self {
		Self { subscription, external_cancellation: None }
	}

	pub fn new_with_cancellation(
		subscription: EventSubscriptionFor<Client, T::Hash>,
		cancellation: Arc<AtomicBool>,
	) -> Self {
		Self { subscription, external_cancellation: Some(cancellation) }
	}

	/// Returns true if a runtime update was detected, false if the wait was cancelled for some other reason
	#[maybe_async::maybe_async(?Send)]
	pub async fn detect_runtime_update(&mut self) -> bool {
		'outer: loop {
			if let Some(canceled) = &self.external_cancellation {
				if canceled.load(Ordering::SeqCst) {
					return false
				}
			}
			let event_records =
				self.subscription.next_events_from_metadata().await.unwrap().unwrap();
			let event_iter = event_records.iter();
			for event in event_iter {
				if event.unwrap().is_code_update() {
					break 'outer
				}
			}
		}
		true
	}
}
