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
	rpc::Request,
};
use ac_compose_macros::rpc_params;
use ac_primitives::{config::Config, AccountInfo};
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use alloc::{string::String, vec::Vec};
use log::*;
use sp_storage::StorageKey;

#[maybe_async::maybe_async(?Send)]
pub trait GetAccountInformation {
	type AccountId;
	type Index;
	type AccountData;

	/// Retrieves the next account index as available on the node.
	async fn get_system_account_next_index(
		&self,
		account_id: Self::AccountId,
	) -> Result<Self::Index>;

	async fn get_account_info(
		&self,
		address: &Self::AccountId,
	) -> Result<Option<AccountInfo<Self::Index, Self::AccountData>>>;

	async fn get_account_data(
		&self,
		address: &Self::AccountId,
	) -> Result<Option<Self::AccountData>>;

	/// Get nonce of an account.
	async fn get_account_nonce(&self, account: &Self::AccountId) -> Result<Self::Index>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> GetAccountInformation for Api<T, Client>
where
	T: Config,
	Client: Request,
{
	type AccountId = T::AccountId;
	type Index = T::Index;
	type AccountData = T::AccountData;

	async fn get_system_account_next_index(
		&self,
		account_id: Self::AccountId,
	) -> Result<Self::Index> {
		let next_index = self
			.client()
			.request("system_accountNextIndex", rpc_params![account_id])
			.await?;
		Ok(next_index)
	}

	async fn get_account_info(
		&self,
		address: &Self::AccountId,
	) -> Result<Option<AccountInfo<Self::Index, Self::AccountData>>> {
		let storagekey: StorageKey = self.metadata().storage_map_key::<Self::AccountId>(
			"System",
			"Account",
			address.clone(),
		)?;

		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key(storagekey, None).await
	}

	async fn get_account_data(
		&self,
		address: &Self::AccountId,
	) -> Result<Option<Self::AccountData>> {
		self.get_account_info(address).await.map(|info| info.map(|i| i.data))
	}

	async fn get_account_nonce(&self, account: &Self::AccountId) -> Result<Self::Index> {
		self.get_account_info(account)
			.await
			.map(|acc_opt| acc_opt.map_or_else(|| 0u32.into(), |acc| acc.nonce))
	}
}

/// Helper functions for some common SystemApi function.
#[maybe_async::maybe_async(?Send)]
pub trait SystemApi {
	type ChainType;
	type Properties;
	type Health;

	/// Get the node's implementation name.
	async fn get_system_name(&self) -> Result<String>;

	/// Get the node implementation's version. Should be a semver string.
	async fn get_system_version(&self) -> Result<String>;

	/// Get the chain's name. Given as a string identifier.
	async fn get_system_chain(&self) -> Result<String>;

	/// Get the chain's type.
	async fn get_system_chain_type(&self) -> Result<Self::ChainType>;

	/// Get a custom set of properties as a JSON object, defined in the chain spec.
	async fn get_system_properties(&self) -> Result<Self::Properties>;

	/// Return health status of the node.
	///
	/// Node is considered healthy if it is:
	/// - connected to some peers (unless running in dev mode)
	/// - not performing a major sync
	async fn get_system_health(&self) -> Result<Self::Health>;

	/// Get the base58-encoded PeerId of the node.
	async fn get_system_local_peer_id(&self) -> Result<String>;

	/// Returns the multi-addresses that the local node is listening on.
	///
	/// The addresses include a trailing `/p2p/` with the local PeerId, and are thus suitable to
	/// be passed to `addReservedPeer` or as a bootnode address for example.
	async fn get_system_local_listen_addresses(&self) -> Result<Vec<String>>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> SystemApi for Api<T, Client>
where
	T: Config,
	Client: Request,
{
	type ChainType = ac_primitives::ChainType;
	type Properties = ac_primitives::Properties;
	type Health = ac_primitives::Health;

	async fn get_system_name(&self) -> Result<String> {
		let res = self.client().request("system_name", rpc_params![]).await?;
		Ok(res)
	}

	async fn get_system_version(&self) -> Result<String> {
		let res = self.client().request("system_version", rpc_params![]).await?;
		Ok(res)
	}

	async fn get_system_chain(&self) -> Result<String> {
		let res = self.client().request("system_chain", rpc_params![]).await?;
		Ok(res)
	}

	async fn get_system_chain_type(&self) -> Result<Self::ChainType> {
		let res = self.client().request("system_chainType", rpc_params![]).await?;
		Ok(res)
	}

	async fn get_system_properties(&self) -> Result<Self::Properties> {
		let res = self.client().request("system_properties", rpc_params![]).await?;
		Ok(res)
	}

	async fn get_system_health(&self) -> Result<Self::Health> {
		let res = self.client().request("system_health", rpc_params![]).await?;
		Ok(res)
	}

	async fn get_system_local_peer_id(&self) -> Result<String> {
		let res = self.client().request("system_localPeerId", rpc_params![]).await?;
		Ok(res)
	}

	async fn get_system_local_listen_addresses(&self) -> Result<Vec<String>> {
		let res = self.client().request("system_localListenAddresses", rpc_params![]).await?;
		Ok(res)
	}
}
