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
use futures::executor::block_on;
use jsonrpsee::core::client::Subscription;
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub struct SubscriptionWrapper<Notification> {
	inner: Subscription<Notification>,
}

// Support async: #278 (careful with no_std compatibility).
impl<Notification: DeserializeOwned> HandleSubscription<Notification>
	for SubscriptionWrapper<Notification>
{
	fn next(&mut self) -> Option<Result<Notification>> {
		block_on(self.inner.next()).map(|result| result.map_err(|e| Error::Client(Box::new(e))))
	}

	fn unsubscribe(self) -> Result<()> {
		block_on(self.inner.unsubscribe()).map_err(|e| Error::Client(Box::new(e)))
	}
}

impl<Notification> From<Subscription<Notification>> for SubscriptionWrapper<Notification> {
	fn from(inner: Subscription<Notification>) -> Self {
		Self { inner }
	}
}
