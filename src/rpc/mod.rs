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

use ac_primitives::RpcParams;
use alloc::string::{String, ToString};
use serde::de::DeserializeOwned;

#[cfg(feature = "ws-client")]
pub use ws_client::WsRpcClient;
#[cfg(feature = "ws-client")]
pub mod ws_client;

#[cfg(feature = "tungstenite-client")]
pub use tungstenite_client::TungsteniteRpcClient;
#[cfg(feature = "tungstenite-client")]
pub mod tungstenite_client;

#[cfg(feature = "jsonrpsee-client")]
pub use jsonrpsee_client::JsonrpseeClient;
#[cfg(feature = "jsonrpsee-client")]
pub mod jsonrpsee_client;

pub mod error;

pub use error::{Error, Result};

#[cfg(test)]
pub mod mocks;

/// Trait to be implemented by the ws-client for sending rpc requests and extrinsic.
pub trait Request {
	/// Sends a RPC request to the substrate node and returns the answer as string.
	fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> Result<R>;
}

/// Trait to be implemented by the ws-client for subscribing to the substrate node.
pub trait Subscribe {
	type Subscription<Notification>: HandleSubscription<Notification>
	where
		Notification: DeserializeOwned;

	fn subscribe<Notification: DeserializeOwned>(
		&self,
		sub: &str,
		params: RpcParams,
		unsub: &str,
	) -> Result<Self::Subscription<Notification>>;
}

/// Trait to use the full functionality of jsonrpseee Subscription type
/// without actually enforcing it.
pub trait HandleSubscription<Notification: DeserializeOwned> {
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

pub fn to_json_req(method: &str, params: RpcParams) -> Result<String> {
	Ok(serde_json::json!({
		"method": method,
		"params": params.to_json_value()?,
		"jsonrpc": "2.0",
		"id": "1",
	})
	.to_string())
}
