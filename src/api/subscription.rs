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

#[cfg(feature = "ws-client")]
use crate::ws_client::client::WsRpcClient;

#[cfg(feature = "tungstenite-client")]
use crate::tungstenite_client::client::TungsteniteRpcClient;

use crate::{
	api::{error::Error, Api, ApiResult, FromHexString},
	rpc::{json_req, RpcClient as RpcClientTrait, Subscriber},
	utils,
};
pub use ac_node_api::{events::EventDetails, StaticEvent};
use ac_node_api::{DispatchError, Events};
use ac_primitives::{BalancesConfig, ExtrinsicParams};
use codec::Decode;
use core::str::FromStr;
use log::*;
use sp_core::Pair;
use sp_rpc::number::NumberOrHex;
use sp_runtime::MultiSigner;
use std::sync::mpsc::{Receiver, Sender as ThreadOut};

#[cfg(feature = "ws-client")]
impl<Signer, Params, Runtime> Api<Signer, WsRpcClient, Params, Runtime>
where
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: BalancesConfig,
	Runtime::Hash: FromHexString,
	Runtime::Balance: TryFrom<NumberOrHex> + FromStr,
	Runtime::Index: Decode,
{
	pub fn default_with_url(url: &str) -> ApiResult<Self> {
		let client = WsRpcClient::new(url);
		Self::new(client)
	}
}

#[cfg(feature = "tungstenite-client")]
impl<Signer, Params, Runtime> Api<Signer, TungsteniteRpcClient, Params, Runtime>
where
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: BalancesConfig,
	Runtime::Hash: FromHexString,
	Runtime::Balance: TryFrom<NumberOrHex> + FromStr,
	Runtime::Index: Decode,
{
	pub fn default_with_url(url: url::Url) -> ApiResult<Self> {
		let client = TungsteniteRpcClient::new(url, 10);
		Self::new(client)
	}
}

impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Signer: Pair,
	MultiSigner: From<Signer::Public>,
	Client: RpcClientTrait + Subscriber,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: BalancesConfig,
	Runtime::Hash: FromHexString,
	Runtime::Balance: TryFrom<NumberOrHex> + FromStr,
{
	pub fn subscribe_events(&self, sender: ThreadOut<String>) -> ApiResult<()> {
		debug!("subscribing to events");
		let key = utils::storage_key("System", "Events");
		let jsonreq = json_req::state_subscribe_storage(vec![key]).to_string();
		self.client().start_subscriber(jsonreq, sender).map_err(|e| e.into())
	}

	pub fn subscribe_finalized_heads(&self, sender: ThreadOut<String>) -> ApiResult<()> {
		debug!("subscribing to finalized heads");
		let jsonreq = json_req::chain_subscribe_finalized_heads().to_string();
		self.client().start_subscriber(jsonreq, sender).map_err(|e| e.into())
	}

	pub fn wait_for_event<Ev: StaticEvent>(&self, receiver: &Receiver<String>) -> ApiResult<Ev> {
		let maybe_event_details = self.wait_for_event_details::<Ev>(receiver)?;
		maybe_event_details
			.as_event()?
			.ok_or(Error::Other("Could not find the specific event".into()))
	}

	pub fn wait_for_event_details<Ev: StaticEvent>(
		&self,
		receiver: &Receiver<String>,
	) -> ApiResult<EventDetails> {
		loop {
			let events_str = receiver.recv()?;
			let event_bytes = Vec::from_hex(events_str)?;
			let events = Events::<Runtime::Hash>::new(
				self.metadata().clone(),
				Default::default(),
				event_bytes,
			);

			for maybe_event_details in events.iter() {
				let event_details = maybe_event_details?;

				// Check for failed xt and return as Dispatch Error in case we find one.
				// Careful - this reports the first one encountered. This event may belong to another extrinsic
				// than the one that is being waited for.
				if extrinsic_has_failed(&event_details) {
					let dispatch_error =
						DispatchError::decode_from(event_details.field_bytes(), self.metadata());
					return Err(Error::Dispatch(dispatch_error))
				}

				let event_metadata = event_details.event_metadata();
				trace!(
					"Found extrinsic: {:?}, {:?}",
					event_metadata.pallet(),
					event_metadata.event()
				);
				if event_metadata.pallet() == Ev::PALLET && event_metadata.event() == Ev::EVENT {
					return Ok(event_details)
				} else {
					trace!("Not the event we are looking for, skipping.")
				}
			}
		}
	}
}

fn extrinsic_has_failed(event_details: &EventDetails) -> bool {
	event_details.pallet_name() == "System" && event_details.variant_name() == "ExtrinsicFailed"
}
