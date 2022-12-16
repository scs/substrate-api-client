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
	utils,
};
use ac_compose_macros::rpc_params;
use ac_primitives::{AccountInfo, ExtrinsicParams, FrameSystemConfig};
use log::*;
use serde::de::DeserializeOwned;
use sp_core::{
	storage::{StorageChangeSet, StorageKey},
	Pair,
};
use sp_runtime::MultiSignature;

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
	Signer: Pair,
	MultiSignature: From<Signer::Signature>,
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
		let key = utils::storage_key("System", "Events");
		self.client()
			.subscribe("state_subscribeStorage", rpc_params![vec![key]], "state_unsubscribeStorage")
			.map_err(|e| e.into())
	}
}
