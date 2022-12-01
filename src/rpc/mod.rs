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
// #[cfg(feature = "ws-client")]
// pub use ws_client::WsRpcClient;

// #[cfg(feature = "ws-client")]
// pub mod ws_client;

pub mod jsonrpsee_client;

pub mod error;
pub mod json_req;

pub use error::*;

use serde::Serialize;
use sp_runtime::DeserializeOwned;

/// Trait to be implemented by the ws-client for sending rpc requests and extrinsic.
pub trait Request {
	/// Sends a RPC request to the substrate node and returns the answer as string.
	fn request<Params: Serialize, R: DeserializeOwned>(
		&self,
		method: &str,
		params: Params,
	) -> Result<R>;
}

/// Trait to be implemented by the ws-client for subscribing to the substrate node.
pub trait Subscribe {
	type Subscription<Notification>: Drop + HandleSubscription<Notification>;

	fn subscribe<Params: Serialize, Notification: DeserializeOwned>(
		&self,
		sub: &str,
		params: Option<Params>,
		unsub: &str,
	) -> Result<Self::Subscription<Notification>>;
}

/// Trait to use the full functionality of jsonrpseee Subscription type
/// without actually enforcing it.
pub trait HandleSubscription<Notification> {
	/// Returns the next notification from the stream.
	/// This may return `None` if the subscription has been terminated,
	/// which may happen if the channel becomes full or is dropped.
	///
	/// **Note:** This has an identical signature to the [`StreamExt::next`]
	/// method (and delegates to that). Import [`StreamExt`] if you'd like
	/// access to other stream combinator methods.
	fn next(&mut self) -> Option<Result<Notification>>;

	/// Unsubscribe and consume the subscription.
	fn unsubscribe(self) -> Result<()>;
}
