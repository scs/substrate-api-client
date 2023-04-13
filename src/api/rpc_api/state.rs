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
	Api, ReadProof,
};
use ac_compose_macros::rpc_params;
use ac_node_api::MetadataError;
use ac_primitives::{
	ExtrinsicParams, FrameSystemConfig, StorageChangeSet, StorageData, StorageKey,
};
use alloc::{string::String, vec, vec::Vec};
use codec::{Decode, Encode};
use log::*;
use serde::de::DeserializeOwned;

pub type StorageChangeSetSubscriptionFor<Client, Hash> =
	<Client as Subscribe>::Subscription<StorageChangeSet<Hash>>;

/// Generic interface to substrate storage.
pub trait GetStorage {
	type Hash;
	/// Retrieve the storage value.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	fn get_storage<V: Decode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>>;

	/// Retrieve the storage value from a map for the given `map_key`.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	fn get_storage_map<K: Encode, V: Decode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		map_key: K,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>>;

	/// Retrieve the key prefix for a storage map. This is the prefix needed for get_storage_keys_paged().
	fn get_storage_map_key_prefix(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
	) -> Result<StorageKey>;

	/// Retrieve the storage value from a double map for the given keys: `first_double_map_key` and `second_double_map_key`.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	fn get_storage_double_map<K: Encode, Q: Encode, V: Decode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		first_double_map_key: K,
		second_double_map_key: Q,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>>;

	/// Retrieve the storage value from the given `storage_key`.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	fn get_storage_by_key<V: Decode>(
		&self,
		storage_key: StorageKey,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>>;

	/// Retrieve the keys with prefix with pagination support.
	/// Up to `count` keys will be returned.
	/// If `start_key` is passed, return next keys in storage in lexicographic order.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	fn get_storage_keys_paged(
		&self,
		prefix: Option<StorageKey>,
		count: u32,
		start_key: Option<StorageKey>,
		at_block: Option<Self::Hash>,
	) -> Result<Vec<StorageKey>>;

	/// Retrieve the raw storage for the given `storage_key`.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	fn get_opaque_storage_by_key(
		&self,
		storage_key: StorageKey,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Vec<u8>>>;

	/// Retrieve the storage proof of the corresponding storage value.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	fn get_storage_value_proof(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>>;

	/// Retrieve the storage proof of the corresponding storage map value.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	fn get_storage_map_proof<K: Encode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		map_key: K,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>>;

	/// Retrieve the storage proof of the corresponding storage double map value.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	fn get_storage_double_map_proof<K: Encode, Q: Encode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		first_double_map_key: K,
		second_double_map_key: Q,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>>;

	/// Retrieve the proof of the corresponding storage entries.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	fn get_storage_proof_by_keys(
		&self,
		keys: Vec<StorageKey>,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>>;

	fn get_keys(
		&self,
		key: StorageKey,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Vec<String>>>;

	fn get_constant<C: Decode>(&self, pallet: &'static str, constant: &'static str) -> Result<C>;
}

impl<Signer, Client, Params, Runtime> GetStorage for Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Runtime: FrameSystemConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
{
	type Hash = Runtime::Hash;

	fn get_storage<V: Decode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>> {
		let storagekey = self.metadata().storage_value_key(pallet, storage_item)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key(storagekey, at_block)
	}

	fn get_storage_map<K: Encode, V: Decode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		map_key: K,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>> {
		let storagekey = self.metadata().storage_map_key::<K>(pallet, storage_item, map_key)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key(storagekey, at_block)
	}

	fn get_storage_map_key_prefix(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
	) -> Result<StorageKey> {
		self.metadata()
			.storage_map_key_prefix(pallet, storage_item)
			.map_err(|e| e.into())
	}

	fn get_storage_double_map<K: Encode, Q: Encode, V: Decode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		first_double_map_key: K,
		second_double_map_key: Q,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>> {
		let storagekey = self.metadata().storage_double_map_key::<K, Q>(
			pallet,
			storage_item,
			first_double_map_key,
			second_double_map_key,
		)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key(storagekey, at_block)
	}

	fn get_storage_by_key<V: Decode>(
		&self,
		storage_key: StorageKey,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>> {
		let s = self.get_opaque_storage_by_key(storage_key, at_block)?;
		match s {
			Some(storage) => Ok(Some(Decode::decode(&mut storage.as_slice())?)),
			None => Ok(None),
		}
	}

	fn get_storage_keys_paged(
		&self,
		storage_key_prefix: Option<StorageKey>,
		count: u32,
		start_key: Option<StorageKey>,
		at_block: Option<Self::Hash>,
	) -> Result<Vec<StorageKey>> {
		let storage = self.client().request(
			"state_getKeysPaged",
			rpc_params![storage_key_prefix, count, start_key, at_block],
		)?;
		Ok(storage)
	}

	fn get_opaque_storage_by_key(
		&self,
		storage_key: StorageKey,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Vec<u8>>> {
		let storage: Option<StorageData> =
			self.client().request("state_getStorage", rpc_params![storage_key, at_block])?;
		Ok(storage.map(|storage_data| storage_data.0))
	}

	fn get_storage_value_proof(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>> {
		let storagekey = self.metadata().storage_value_key(pallet, storage_item)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_proof_by_keys(vec![storagekey], at_block)
	}

	fn get_storage_map_proof<K: Encode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		map_key: K,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>> {
		let storagekey = self.metadata().storage_map_key::<K>(pallet, storage_item, map_key)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_proof_by_keys(vec![storagekey], at_block)
	}

	fn get_storage_double_map_proof<K: Encode, Q: Encode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		first_double_map_key: K,
		second_double_map_key: Q,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>> {
		let storage_key = self.metadata().storage_double_map_key::<K, Q>(
			pallet,
			storage_item,
			first_double_map_key,
			second_double_map_key,
		)?;
		info!("storage key is: 0x{}", hex::encode(&storage_key));
		self.get_storage_proof_by_keys(vec![storage_key], at_block)
	}

	fn get_storage_proof_by_keys(
		&self,
		storage_keys: Vec<StorageKey>,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>> {
		let proof = self
			.client()
			.request("state_getReadProof", rpc_params![storage_keys, at_block])?;
		Ok(proof)
	}

	fn get_keys(
		&self,
		storage_key: StorageKey,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Vec<String>>> {
		let keys = self.client().request("state_getKeys", rpc_params![storage_key, at_block])?;
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
pub trait SubscribeState {
	type Client: Subscribe;
	type Hash: DeserializeOwned;

	fn subscribe_state(
		&self,
		pallet: &str,
		storage_key: &str,
	) -> Result<StorageChangeSetSubscriptionFor<Self::Client, Self::Hash>>;
}

impl<Signer, Client, Params, Runtime> SubscribeState for Api<Signer, Client, Params, Runtime>
where
	Client: Subscribe,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	type Client = Client;
	type Hash = Runtime::Hash;

	fn subscribe_state(
		&self,
		pallet: &str,
		storage_key_name: &str,
	) -> Result<StorageChangeSetSubscriptionFor<Self::Client, Self::Hash>> {
		debug!("subscribing to events");
		let key = crate::storage_key(pallet, storage_key_name);
		self.client()
			.subscribe("state_subscribeStorage", rpc_params![vec![key]], "state_unsubscribeStorage")
			.map_err(|e| e.into())
	}
}
