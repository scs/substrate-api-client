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

use crate::rpc::{Error, HandleSubscription, Result};
use core::marker::PhantomData;
use serde::de::DeserializeOwned;
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub struct TungsteniteSubscriptionWrapper<Notification> {
	receiver: Receiver<String>,
	_phantom: PhantomData<Notification>,
}

impl<Notification> TungsteniteSubscriptionWrapper<Notification> {
	pub fn new(receiver: Receiver<String>) -> Self {
		Self { receiver, _phantom: Default::default() }
	}
}

#[maybe_async::maybe_async(?Send)]
impl<Notification: DeserializeOwned> HandleSubscription<Notification>
	for TungsteniteSubscriptionWrapper<Notification>
{
	async fn next(&mut self) -> Option<Result<Notification>> {
		let notification = match self.receiver.recv() {
			Ok(notif) => notif,
			// Sender was disconnected, therefore no further messages are to be expected.
			Err(_e) => return None,
		};
		Some(serde_json::from_str(&notification).map_err(|_| Error::ExtrinsicFailed(notification)))
	}

	async fn unsubscribe(self) -> Result<()> {
		// TODO: Nicer unsubscription.
		// We close ungracefully: Simply drop the receiver. This will turn
		// into an error on the sender side, terminating the websocket polling
		// loop.
		Ok(())
	}
}
