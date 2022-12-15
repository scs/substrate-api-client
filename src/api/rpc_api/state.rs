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
use crate::{
	api::Result,
	rpc::{Request, Subscribe},
	utils, Api, MetadataError, ReadProof,
};
use ac_compose_macros::rpc_params;
use ac_primitives::{ExtrinsicParams, FrameSystemConfig};
use codec::{Decode, Encode};
use log::*;
use serde::de::DeserializeOwned;
use sp_core::storage::{StorageChangeSet, StorageData, StorageKey};

/// Generic interface to substrate storage.
pub trait GetStorage<Hash> {
	fn get_storage_value<V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Hash>,
	) -> Result<Option<V>>;

	fn get_storage_map<K: Encode, V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		map_key: K,
		at_block: Option<Hash>,
	) -> Result<Option<V>>;

	fn get_storage_map_key_prefix(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
	) -> Result<StorageKey>;

	fn get_storage_double_map<K: Encode, Q: Encode, V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
		second: Q,
		at_block: Option<Hash>,
	) -> Result<Option<V>>;

	fn get_storage_by_key_hash<V: Decode>(
		&self,
		key: StorageKey,
		at_block: Option<Hash>,
	) -> Result<Option<V>>;

	fn get_opaque_storage_by_key_hash(
		&self,
		key: StorageKey,
		at_block: Option<Hash>,
	) -> Result<Option<Vec<u8>>>;

	fn get_storage_value_proof(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Hash>,
	) -> Result<Option<ReadProof<Hash>>>;

	fn get_storage_map_proof<K: Encode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		map_key: K,
		at_block: Option<Hash>,
	) -> Result<Option<ReadProof<Hash>>>;

	fn get_storage_double_map_proof<K: Encode, Q: Encode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
		second: Q,
		at_block: Option<Hash>,
	) -> Result<Option<ReadProof<Hash>>>;

	fn get_storage_proof_by_keys(
		&self,
		keys: Vec<StorageKey>,
		at_block: Option<Hash>,
	) -> Result<Option<ReadProof<Hash>>>;

	fn get_keys(&self, key: StorageKey, at_block: Option<Hash>) -> Result<Option<Vec<String>>>;

	fn get_constant<C: Decode>(&self, pallet: &'static str, constant: &'static str) -> Result<C>;
}

impl<Signer, Client, Params, Runtime> GetStorage<Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Runtime: FrameSystemConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
{
	fn get_storage_value<V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Runtime::Hash>,
	) -> Result<Option<V>> {
		let storagekey = self.metadata().storage_value_key(storage_prefix, storage_key_name)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key_hash(storagekey, at_block)
	}

	fn get_storage_map<K: Encode, V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		map_key: K,
		at_block: Option<Runtime::Hash>,
	) -> Result<Option<V>> {
		let storagekey =
			self.metadata()
				.storage_map_key::<K>(storage_prefix, storage_key_name, map_key)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key_hash(storagekey, at_block)
	}

	fn get_storage_map_key_prefix(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
	) -> Result<StorageKey> {
		self.metadata()
			.storage_map_key_prefix(storage_prefix, storage_key_name)
			.map_err(|e| e.into())
	}

	fn get_storage_double_map<K: Encode, Q: Encode, V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
		second: Q,
		at_block: Option<Runtime::Hash>,
	) -> Result<Option<V>> {
		let storagekey = self.metadata().storage_double_map_key::<K, Q>(
			storage_prefix,
			storage_key_name,
			first,
			second,
		)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key_hash(storagekey, at_block)
	}

	fn get_storage_by_key_hash<V: Decode>(
		&self,
		key: StorageKey,
		at_block: Option<Runtime::Hash>,
	) -> Result<Option<V>> {
		let s = self.get_opaque_storage_by_key_hash(key, at_block)?;
		match s {
			Some(storage) => Ok(Some(Decode::decode(&mut storage.as_slice())?)),
			None => Ok(None),
		}
	}

	fn get_opaque_storage_by_key_hash(
		&self,
		key: StorageKey,
		at_block: Option<Runtime::Hash>,
	) -> Result<Option<Vec<u8>>> {
		let storage: Option<StorageData> =
			self.client().request("state_getStorage", rpc_params![key, at_block])?;
		Ok(storage.map(|storage_data| storage_data.0))
	}

	fn get_storage_value_proof(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Runtime::Hash>,
	) -> Result<Option<ReadProof<Runtime::Hash>>> {
		let storagekey = self.metadata().storage_value_key(storage_prefix, storage_key_name)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_proof_by_keys(vec![storagekey], at_block)
	}

	fn get_storage_map_proof<K: Encode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		map_key: K,
		at_block: Option<Runtime::Hash>,
	) -> Result<Option<ReadProof<Runtime::Hash>>> {
		let storagekey =
			self.metadata()
				.storage_map_key::<K>(storage_prefix, storage_key_name, map_key)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_proof_by_keys(vec![storagekey], at_block)
	}

	fn get_storage_double_map_proof<K: Encode, Q: Encode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
		second: Q,
		at_block: Option<Runtime::Hash>,
	) -> Result<Option<ReadProof<Runtime::Hash>>> {
		let storagekey = self.metadata().storage_double_map_key::<K, Q>(
			storage_prefix,
			storage_key_name,
			first,
			second,
		)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_proof_by_keys(vec![storagekey], at_block)
	}

	fn get_storage_proof_by_keys(
		&self,
		keys: Vec<StorageKey>,
		at_block: Option<Runtime::Hash>,
	) -> Result<Option<ReadProof<Runtime::Hash>>> {
		let proof = self.client().request("state_getReadProof", rpc_params![keys, at_block])?;
		Ok(proof)
	}

	fn get_keys(
		&self,
		key: StorageKey,
		at_block: Option<Runtime::Hash>,
	) -> Result<Option<Vec<String>>> {
		let keys = self.client().request("state_getKeys", rpc_params![key, at_block])?;
		Ok(keys)
	}

	fn get_constant<C: Decode>(&self, pallet: &'static str, constant: &'static str) -> Result<C> {
		let c = self
			.metadata()
			.pallet(pallet)?
			.constants
			.get(constant)
			.ok_or(MetadataError::ConstantNotFound(constant))?;

		Ok(Decode::decode(&mut c.value.as_slice())?)
	}
}

pub trait SubscribeState<Client, Hash>
where
	Client: Subscribe,
	Hash: DeserializeOwned,
{
	fn subscribe_state(
		&self,
		pallet: &str,
		storage_key: &str,
	) -> Result<Client::Subscription<StorageChangeSet<Hash>>>;
}

impl<Signer, Client, Params, Runtime> SubscribeState<Client, Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Client: Subscribe,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	fn subscribe_state(
		&self,
		pallet: &str,
		storage_key_name: &str,
	) -> Result<Client::Subscription<StorageChangeSet<Runtime::Hash>>> {
		debug!("subscribing to events");
		let key = utils::storage_key(pallet, storage_key_name);
		self.client()
			.subscribe("state_subscribeStorage", rpc_params![vec![key]], "state_unsubscribeStorage")
			.map_err(|e| e.into())
	}
}
