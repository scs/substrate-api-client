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

//! Interface to common frame system pallet information.

use crate::{
	api::{Api, GetStorage, Result},
	rpc::{Request, Subscribe},
};
use ac_compose_macros::rpc_params;
use ac_primitives::{
	AccountInfo, ExtrinsicParams, FrameSystemConfig, SignExtrinsic, StorageChangeSet, StorageKey,
};
use alloc::{string::String, vec, vec::Vec};
use log::*;
use serde::de::DeserializeOwned;

pub trait GetAccountInformation<AccountId> {
	type Index;
	type AccountData;

	fn get_account_info(
		&self,
		address: &AccountId,
	) -> Result<Option<AccountInfo<Self::Index, Self::AccountData>>>;

	fn get_account_data(&self, address: &AccountId) -> Result<Option<Self::AccountData>>;
}

impl<Signer, Client, Params, Runtime> GetAccountInformation<Runtime::AccountId>
	for Api<Signer, Client, Params, Runtime>
where
	Signer: SignExtrinsic<Runtime::AccountId>,
	Client: Request,
	Runtime: FrameSystemConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
{
	type Index = Runtime::Index;
	type AccountData = Runtime::AccountData;

	fn get_account_info(
		&self,
		address: &Runtime::AccountId,
	) -> Result<Option<AccountInfo<Self::Index, Self::AccountData>>> {
		let storagekey: StorageKey = self.metadata().storage_map_key::<Runtime::AccountId>(
			"System",
			"Account",
			address.clone(),
		)?;

		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key_hash(storagekey, None)
	}

	fn get_account_data(
		&self,
		address: &Runtime::AccountId,
	) -> Result<Option<Runtime::AccountData>> {
		self.get_account_info(address).map(|info| info.map(|i| i.data))
	}
}

/// Helper functions for some common SystemApi function.
pub trait SystemApi {
	type ChainType;
	type Properties;
	type Health;

	/// Get the node's implementation name.
	fn get_system_name(&self) -> Result<String>;

	/// Get the node implementation's version. Should be a semver string.
	fn get_system_version(&self) -> Result<String>;

	/// Get the chain's name. Given as a string identifier.
	fn get_system_chain(&self) -> Result<String>;

	/// Get the chain's type.
	fn get_system_chain_type(&self) -> Result<Self::ChainType>;

	/// Get a custom set of properties as a JSON object, defined in the chain spec.
	fn get_system_properties(&self) -> Result<Self::Properties>;

	/// Return health status of the node.
	///
	/// Node is considered healthy if it is:
	/// - connected to some peers (unless running in dev mode)
	/// - not performing a major sync
	fn get_system_health(&self) -> Result<Self::Health>;

	/// Get the base58-encoded PeerId of the node.
	fn get_system_local_peer_id(&self) -> Result<String>;

	/// Returns the multi-addresses that the local node is listening on
	///
	/// The addresses include a trailing `/p2p/` with the local PeerId, and are thus suitable to
	/// be passed to `addReservedPeer` or as a bootnode address for example.
	fn get_system_local_listen_addresses(&self) -> Result<Vec<String>>;
}

impl<Signer, Client, Params, Runtime> SystemApi for Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Runtime: FrameSystemConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
{
	type ChainType = ac_primitives::ChainType;
	type Properties = ac_primitives::Properties;
	type Health = ac_primitives::Health;

	fn get_system_name(&self) -> Result<String> {
		let res = self.client().request("system_name", rpc_params![])?;
		Ok(res)
	}

	fn get_system_version(&self) -> Result<String> {
		let res = self.client().request("system_version", rpc_params![])?;
		Ok(res)
	}

	fn get_system_chain(&self) -> Result<String> {
		let res = self.client().request("system_chain", rpc_params![])?;
		Ok(res)
	}

	fn get_system_chain_type(&self) -> Result<Self::ChainType> {
		let res = self.client().request("system_chainType", rpc_params![])?;
		Ok(res)
	}

	fn get_system_properties(&self) -> Result<Self::Properties> {
		let res = self.client().request("system_properties", rpc_params![])?;
		Ok(res)
	}

	fn get_system_health(&self) -> Result<Self::Health> {
		let res = self.client().request("system_health", rpc_params![])?;
		Ok(res)
	}

	fn get_system_local_peer_id(&self) -> Result<String> {
		let res = self.client().request("system_localPeerId", rpc_params![])?;
		Ok(res)
	}

	fn get_system_local_listen_addresses(&self) -> Result<Vec<String>> {
		let res = self.client().request("system_localListenAddresses", rpc_params![])?;
		Ok(res)
	}
}

pub trait SubscribeFrameSystem<Client, Hash>
where
	Client: Subscribe,
	Hash: DeserializeOwned,
{
	fn subscribe_system_events(&self) -> Result<Client::Subscription<StorageChangeSet<Hash>>>;
}

impl<Signer, Client, Params, Runtime> SubscribeFrameSystem<Client, Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Client: Subscribe,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	fn subscribe_system_events(
		&self,
	) -> Result<Client::Subscription<StorageChangeSet<Runtime::Hash>>> {
		debug!("subscribing to events");
		let key = crate::storage_key("System", "Events");
		self.client()
			.subscribe("state_subscribeStorage", rpc_params![vec![key]], "state_unsubscribeStorage")
			.map_err(|e| e.into())
	}
}
