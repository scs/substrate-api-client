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
use ac_primitives::config::Config;
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};
use codec::{Decode, Encode};
use core::cmp;
use log::*;
use serde::de::DeserializeOwned;
use sp_storage::{StorageChangeSet, StorageData, StorageKey};

/// Default substrate value of maximum number of keys returned.
// See https://github.com/paritytech/substrate/blob/9f6fecfeea15345c983629af275b1f1702a50004/client/rpc/src/state/mod.rs#L54
const STORAGE_KEYS_PAGED_MAX_COUNT: u32 = 1000;

pub type StorageChangeSetSubscriptionFor<Client, Hash> =
	<Client as Subscribe>::Subscription<StorageChangeSet<Hash>>;

/// Generic interface to substrate storage.
#[maybe_async::maybe_async(?Send)]
pub trait GetStorage {
	type Hash;
	/// Retrieve the storage value.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	async fn get_storage<V: Decode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>>;

	/// Retrieve the storage value from a map for the given `map_key`.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	async fn get_storage_map<K: Encode, V: Decode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		map_key: K,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>>;

	/// Retrieve the key prefix for a storage map. This is the prefix needed for get_storage_keys_paged().
	async fn get_storage_map_key_prefix(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
	) -> Result<StorageKey>;

	async fn get_storage_double_map_key_prefix<K: Encode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
	) -> Result<StorageKey>;

	/// Retrieve the storage value from a double map for the given keys: `first_double_map_key` and `second_double_map_key`.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	async fn get_storage_double_map<K: Encode, Q: Encode, V: Decode>(
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
	async fn get_storage_by_key<V: Decode>(
		&self,
		storage_key: StorageKey,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>>;

	/// Retrieve the keys with prefix with pagination support.
	/// Call the RPC substrate storage_keys_paged, which limits the number of returned keys.
	///
	/// Up to `count` keys will be returned. If `count` is too big, an error will be returned
	/// If `start_key` is passed, return next keys in storage in lexicographic order.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	// See https://github.com/paritytech/substrate/blob/9f6fecfeea15345c983629af275b1f1702a50004/client/rpc/src/state/mod.rs#L54
	async fn get_storage_keys_paged_limited(
		&self,
		prefix: Option<StorageKey>,
		count: u32,
		start_key: Option<StorageKey>,
		at_block: Option<Self::Hash>,
	) -> Result<Vec<StorageKey>>;

	/// Retrieve up to `count` keys. Support prefix and pagination support.
	/// The number of keys returned is not limited. For big numbers, the rpc calls will be made several times.
	/// Up to `count` keys will be returned.
	/// If `start_key` is passed, return next keys in storage in lexicographic order.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	async fn get_storage_keys_paged(
		&self,
		prefix: Option<StorageKey>,
		count: u32,
		start_key: Option<StorageKey>,
		at_block: Option<Self::Hash>,
	) -> Result<Vec<StorageKey>>;

	/// Retrieve the raw storage for the given `storage_key`.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	async fn get_opaque_storage_by_key(
		&self,
		storage_key: StorageKey,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Vec<u8>>>;

	/// Retrieve the storage proof of the corresponding storage value.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	async fn get_storage_value_proof(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>>;

	/// Retrieve the storage proof of the corresponding storage map value.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	async fn get_storage_map_proof<K: Encode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		map_key: K,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>>;

	/// Retrieve the storage proof of the corresponding storage double map value.
	///
	/// `at_block`: the state is queried at this block, set to `None` to get the state from the latest known block.
	async fn get_storage_double_map_proof<K: Encode, Q: Encode>(
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
	async fn get_storage_proof_by_keys(
		&self,
		keys: Vec<StorageKey>,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>>;

	async fn get_keys(
		&self,
		key: StorageKey,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Vec<String>>>;

	async fn get_constant<C: Decode>(
		&self,
		pallet: &'static str,
		constant: &'static str,
	) -> Result<C>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> GetStorage for Api<T, Client>
where
	T: Config,
	Client: Request,
{
	type Hash = T::Hash;

	async fn get_storage<V: Decode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>> {
		let storagekey = self.metadata().storage_value_key(pallet, storage_item)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key(storagekey, at_block).await
	}

	async fn get_storage_map<K: Encode, V: Decode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		map_key: K,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>> {
		let storagekey = self.metadata().storage_map_key::<K>(pallet, storage_item, map_key)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key(storagekey, at_block).await
	}

	async fn get_storage_map_key_prefix(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
	) -> Result<StorageKey> {
		self.metadata()
			.storage_map_key_prefix(pallet, storage_item)
			.map_err(|e| e.into())
	}

	async fn get_storage_double_map<K: Encode, Q: Encode, V: Decode>(
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
		self.get_storage_by_key(storagekey, at_block).await
	}

	async fn get_storage_double_map_key_prefix<K: Encode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
	) -> Result<StorageKey> {
		self.metadata()
			.storage_double_map_key_prefix(storage_prefix, storage_key_name, first)
			.map_err(|e| e.into())
	}

	async fn get_storage_by_key<V: Decode>(
		&self,
		storage_key: StorageKey,
		at_block: Option<Self::Hash>,
	) -> Result<Option<V>> {
		let s = self.get_opaque_storage_by_key(storage_key, at_block).await?;
		match s {
			Some(storage) => Ok(Some(Decode::decode(&mut storage.as_slice())?)),
			None => Ok(None),
		}
	}

	async fn get_storage_keys_paged_limited(
		&self,
		storage_key_prefix: Option<StorageKey>,
		count: u32,
		start_key: Option<StorageKey>,
		at_block: Option<Self::Hash>,
	) -> Result<Vec<StorageKey>> {
		let storage = self
			.client()
			.request(
				"state_getKeysPaged",
				rpc_params![storage_key_prefix, count, start_key, at_block],
			)
			.await?;
		Ok(storage)
	}

	async fn get_storage_keys_paged(
		&self,
		storage_key_prefix: Option<StorageKey>,
		count: u32,
		start_key: Option<StorageKey>,
		at_block: Option<Self::Hash>,
	) -> Result<Vec<StorageKey>> {
		let mut storage_keys: Vec<StorageKey> = Vec::new();
		let mut keys_left_to_fetch = count;
		let mut new_start_key = start_key;

		while keys_left_to_fetch > 0 {
			let new_count = cmp::min(STORAGE_KEYS_PAGED_MAX_COUNT, keys_left_to_fetch);
			let mut keys = self
				.get_storage_keys_paged_limited(
					storage_key_prefix.clone(),
					new_count,
					new_start_key,
					at_block,
				)
				.await?;
			let num_keys = keys.len() as u32;
			storage_keys.append(&mut keys);
			if num_keys < new_count {
				break
			}
			keys_left_to_fetch -= new_count;
			new_start_key = keys.last().map(|x| x.to_owned());
		}

		Ok(storage_keys)
	}

	async fn get_opaque_storage_by_key(
		&self,
		storage_key: StorageKey,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Vec<u8>>> {
		let storage: Option<StorageData> = self
			.client()
			.request("state_getStorage", rpc_params![storage_key, at_block])
			.await?;
		Ok(storage.map(|storage_data| storage_data.0))
	}

	async fn get_storage_value_proof(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>> {
		let storagekey = self.metadata().storage_value_key(pallet, storage_item)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_proof_by_keys(vec![storagekey], at_block).await
	}

	async fn get_storage_map_proof<K: Encode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		map_key: K,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>> {
		let storagekey = self.metadata().storage_map_key::<K>(pallet, storage_item, map_key)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_proof_by_keys(vec![storagekey], at_block).await
	}

	async fn get_storage_double_map_proof<K: Encode, Q: Encode>(
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
		self.get_storage_proof_by_keys(vec![storage_key], at_block).await
	}

	async fn get_storage_proof_by_keys(
		&self,
		storage_keys: Vec<StorageKey>,
		at_block: Option<Self::Hash>,
	) -> Result<Option<ReadProof<Self::Hash>>> {
		let proof = self
			.client()
			.request("state_getReadProof", rpc_params![storage_keys, at_block])
			.await?;
		Ok(proof)
	}

	async fn get_keys(
		&self,
		storage_key: StorageKey,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Vec<String>>> {
		let keys = self
			.client()
			.request("state_getKeys", rpc_params![storage_key, at_block])
			.await?;
		Ok(keys)
	}

	async fn get_constant<C: Decode>(
		&self,
		pallet: &'static str,
		constant: &'static str,
	) -> Result<C> {
		let c = self
			.metadata()
			.pallet_by_name_err(pallet)?
			.constant_by_name(constant)
			.ok_or(MetadataError::ConstantNotFound(constant))?;

		Ok(Decode::decode(&mut c.value.as_slice())?)
	}
}

#[maybe_async::maybe_async(?Send)]
pub trait SubscribeState {
	type Client: Subscribe;
	type Hash: DeserializeOwned;

	async fn subscribe_state(
		&self,
		pallet: &str,
		storage_key: &str,
	) -> Result<StorageChangeSetSubscriptionFor<Self::Client, Self::Hash>>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> SubscribeState for Api<T, Client>
where
	T: Config,
	Client: Subscribe,
{
	type Client = Client;
	type Hash = T::Hash;

	async fn subscribe_state(
		&self,
		pallet: &str,
		storage_key_name: &str,
	) -> Result<StorageChangeSetSubscriptionFor<Self::Client, Self::Hash>> {
		debug!("subscribing to events");
		let key = crate::storage_key(pallet, storage_key_name);
		self.client()
			.subscribe("state_subscribeStorage", rpc_params![vec![key]], "state_unsubscribeStorage")
			.await
			.map_err(|e| e.into())
	}
}
