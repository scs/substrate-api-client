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

use crate::rpc::{HandleSubscription, Result};
use core::marker::PhantomData;
use serde::de::DeserializeOwned;
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub struct WsSubscriptionWrapper<Notification> {
	receiver: Receiver<String>,
	_phantom: PhantomData<Notification>,
}

impl<Notification> WsSubscriptionWrapper<Notification> {
	pub fn new(receiver: Receiver<String>) -> Self {
		Self { receiver, _phantom: Default::default() }
	}
}

// Support async: #278 (careful with no_std compatibility).
impl<Notification: DeserializeOwned> HandleSubscription<Notification>
	for WsSubscriptionWrapper<Notification>
{
	fn next(&mut self) -> Option<Result<Notification>> {
		let notification = match self.receiver.recv() {
			Ok(notif) => notif,
			// Sender was disconnected, therefore no further messages are to be expected.
			Err(_) => return None,
		};
		// Decode to the notification on is expecting.
		let result = serde_json::from_str(&notification).map_err(|e| e.into());
		Some(result)
	}

	fn unsubscribe(self) -> Result<()> {
		core::mem::drop(self.receiver);
		Ok(())
	}
}
