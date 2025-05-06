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
use ws::Sender as WsSender;

#[derive(Debug)]
pub struct WsSubscriptionWrapper<Notification> {
	ws_sender: WsSender,
	receiver: Receiver<String>,
	_phantom: PhantomData<Notification>,
}

impl<Notification> WsSubscriptionWrapper<Notification> {
	pub fn new(ws_sender: WsSender, receiver: Receiver<String>) -> Self {
		Self { ws_sender, receiver, _phantom: Default::default() }
	}
}

#[maybe_async::maybe_async(?Send)]
impl<Notification: DeserializeOwned> HandleSubscription<Notification>
	for WsSubscriptionWrapper<Notification>
{
	async fn next(&mut self) -> Option<Result<Notification>> {
		let notification = match self.receiver.recv() {
			Ok(notif) => notif,
			// Sender was disconnected, therefore no further messages are to be expected.
			Err(_) => return None,
		};
		Some(serde_json::from_str(&notification).map_err(|_| Error::ExtrinsicFailed(notification)))
	}

	async fn unsubscribe(self) -> Result<()> {
		self.ws_sender.clone().shutdown()?;
		Ok(())
	}
}

impl<Notification> Drop for WsSubscriptionWrapper<Notification> {
	fn drop(&mut self) {
		if let Err(e) = self.ws_sender.shutdown() {
			log::error!("Could not properly shutdown websocket connection due to {:?}", e);
		}
	}
}
