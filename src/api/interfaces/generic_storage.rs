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
	api::ApiResult, rpc::json_req, utils::FromHexString, Api, ExtrinsicParams,
	MetadataError, ReadProof, RpcClient,
};
use ac_primitives::FrameSystemConfig;
use codec::{Decode, Encode};
use log::*;
use sp_core::storage::StorageKey;

/// Generic interface to substrate storage.
pub trait GetGenericStorageInterface<Hash> {
	fn get_storage_value<V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Hash>,
	) -> ApiResult<Option<V>>;

	fn get_storage_map<K: Encode, V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		map_key: K,
		at_block: Option<Hash>,
	) -> ApiResult<Option<V>>;

	fn get_storage_map_key_prefix(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
	) -> ApiResult<StorageKey>;

	fn get_storage_double_map<K: Encode, Q: Encode, V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
		second: Q,
		at_block: Option<Hash>,
	) -> ApiResult<Option<V>>;

	fn get_storage_by_key_hash<V: Decode>(
		&self,
		key: StorageKey,
		at_block: Option<Hash>,
	) -> ApiResult<Option<V>>;

	fn get_opaque_storage_by_key_hash(
		&self,
		key: StorageKey,
		at_block: Option<Hash>,
	) -> ApiResult<Option<Vec<u8>>>;

	fn get_storage_value_proof(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Hash>,
	) -> ApiResult<Option<ReadProof<Hash>>>;

	fn get_storage_map_proof<K: Encode, V: Decode + Clone>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		map_key: K,
		at_block: Option<Hash>,
	) -> ApiResult<Option<ReadProof<Hash>>>;

	fn get_storage_double_map_proof<K: Encode, Q: Encode, V: Decode + Clone>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
		second: Q,
		at_block: Option<Hash>,
	) -> ApiResult<Option<ReadProof<Hash>>>;

	fn get_storage_proof_by_keys(
		&self,
		keys: Vec<StorageKey>,
		at_block: Option<Hash>,
	) -> ApiResult<Option<ReadProof<Hash>>>;

	fn get_keys(&self, key: StorageKey, at_block: Option<Hash>) -> ApiResult<Option<Vec<String>>>;

	fn get_constant<C: Decode>(&self, pallet: &'static str, constant: &'static str)
		-> ApiResult<C>;
}

impl<Signer, Client, Params, Runtime> GetGenericStorageInterface<Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Client: RpcClient,
	Runtime: FrameSystemConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
{
	fn get_storage_value<V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<V>> {
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
	) -> ApiResult<Option<V>> {
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
	) -> ApiResult<StorageKey> {
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
	) -> ApiResult<Option<V>> {
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
	) -> ApiResult<Option<V>> {
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
	) -> ApiResult<Option<Vec<u8>>> {
		let jsonreq = json_req::state_get_storage(key, at_block);
		let s = self.client().get_request(jsonreq)?;

		match s {
			Some(storage) => Ok(Some(Vec::from_hex(storage)?)),
			None => Ok(None),
		}
	}
	fn get_storage_value_proof(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<ReadProof<Runtime::Hash>>> {
		let storagekey = self.metadata().storage_value_key(storage_prefix, storage_key_name)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_proof_by_keys(vec![storagekey], at_block)
	}

	fn get_storage_map_proof<K: Encode, V: Decode + Clone>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		map_key: K,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<ReadProof<Runtime::Hash>>> {
		let storagekey =
			self.metadata()
				.storage_map_key::<K>(storage_prefix, storage_key_name, map_key)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_proof_by_keys(vec![storagekey], at_block)
	}

	fn get_storage_double_map_proof<K: Encode, Q: Encode, V: Decode + Clone>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
		second: Q,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<ReadProof<Runtime::Hash>>> {
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
	) -> ApiResult<Option<ReadProof<Runtime::Hash>>> {
		let jsonreq = json_req::state_get_read_proof(keys, at_block);
		let p = self.client().get_request(jsonreq)?;
		match p {
			Some(proof) => Ok(Some(serde_json::from_str(&proof)?)),
			None => Ok(None),
		}
	}

	fn get_keys(
		&self,
		key: StorageKey,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<Vec<String>>> {
		let jsonreq = json_req::state_get_keys(key, at_block);
		let k = self.client().get_request(jsonreq)?;
		match k {
			Some(keys) => Ok(Some(serde_json::from_str(&keys)?)),
			None => Ok(None),
		}
	}

	fn get_constant<C: Decode>(
		&self,
		pallet: &'static str,
		constant: &'static str,
	) -> ApiResult<C> {
		let c = self
			.metadata()
			.pallet(pallet)?
			.constants
			.get(constant)
			.ok_or(MetadataError::ConstantNotFound(constant))?;

		Ok(Decode::decode(&mut c.value.as_slice())?)
	}
}
