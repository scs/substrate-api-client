/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG
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

//! For querying runtime storage.

use crate::metadata::MetadataError;
use alloc::{borrow::ToOwned, vec::Vec};
use codec::Encode;
use core::marker::PhantomData;
use frame_metadata::v15::{StorageEntryMetadata, StorageEntryType, StorageHasher};
use scale_info::form::PortableForm;
use sp_storage::StorageKey;

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct StorageValue {
	module_prefix: Vec<u8>,
	storage_prefix: Vec<u8>,
}

impl StorageValue {
	pub fn key(&self) -> StorageKey {
		let mut bytes = sp_crypto_hashing::twox_128(&self.module_prefix).to_vec();
		bytes.extend(&sp_crypto_hashing::twox_128(&self.storage_prefix)[..]);
		StorageKey(bytes)
	}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StorageMap<K> {
	_marker: PhantomData<K>,
	module_prefix: Vec<u8>,
	storage_prefix: Vec<u8>,
	hasher: StorageHasher,
}

impl<K: Encode> StorageMap<K> {
	pub fn key(&self, key: K) -> StorageKey {
		let mut bytes = sp_crypto_hashing::twox_128(&self.module_prefix).to_vec();
		bytes.extend(&sp_crypto_hashing::twox_128(&self.storage_prefix)[..]);
		bytes.extend(key_hash(&key, &self.hasher));
		StorageKey(bytes)
	}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StorageDoubleMap<K, Q> {
	_marker: PhantomData<K>,
	_marker2: PhantomData<Q>,
	module_prefix: Vec<u8>,
	storage_prefix: Vec<u8>,
	hasher: StorageHasher,
	key2_hasher: StorageHasher,
}

impl<K: Encode, Q: Encode> StorageDoubleMap<K, Q> {
	pub fn key(&self, key1: K, key2: Q) -> StorageKey {
		let mut bytes = sp_crypto_hashing::twox_128(&self.module_prefix).to_vec();
		bytes.extend(&sp_crypto_hashing::twox_128(&self.storage_prefix)[..]);
		bytes.extend(key_hash(&key1, &self.hasher));
		bytes.extend(key_hash(&key2, &self.key2_hasher));
		StorageKey(bytes)
	}
}

/// trait to extract the storage based on the [`StorageEntryMetadata`].
pub trait GetStorageTypes {
	fn get_double_map<K: Encode, Q: Encode>(
		&self,
		pallet_prefix: &str,
	) -> Result<StorageDoubleMap<K, Q>, MetadataError>;
	fn get_map<K: Encode>(&self, pallet_prefix: &str) -> Result<StorageMap<K>, MetadataError>;
	fn get_map_prefix(&self, pallet_prefix: &str) -> Result<StorageKey, MetadataError>;
	fn get_value(&self, pallet_prefix: &str) -> Result<StorageValue, MetadataError>;
	fn get_double_map_prefix<K: Encode>(
		&self,
		pallet_prefix: &str,
		key1: K,
	) -> Result<StorageKey, MetadataError>;
}

impl GetStorageTypes for StorageEntryMetadata<PortableForm> {
	fn get_double_map<K: Encode, Q: Encode>(
		&self,
		pallet_prefix: &str,
	) -> Result<StorageDoubleMap<K, Q>, MetadataError> {
		match &self.ty {
			StorageEntryType::Map { hashers, .. } => {
				let module_prefix = pallet_prefix.as_bytes().to_vec();
				let storage_prefix = self.name.as_bytes().to_vec();
				let hasher1 = hashers.first().ok_or(MetadataError::StorageTypeError)?;
				let hasher2 = hashers.get(1).ok_or(MetadataError::StorageTypeError)?;

				// hashers do not implement debug in no_std
				#[cfg(feature = "std")]
				log::debug!(
					"map for '{}' '{}' has hasher1 {:?} hasher2 {:?}",
					pallet_prefix,
					self.name,
					hasher1,
					hasher2
				);

				Ok(StorageDoubleMap {
					_marker: PhantomData,
					_marker2: PhantomData,
					module_prefix,
					storage_prefix,
					hasher: hasher1.to_owned(),
					key2_hasher: hasher2.to_owned(),
				})
			},
			_ => Err(MetadataError::StorageTypeError),
		}
	}
	fn get_map<K: Encode>(&self, pallet_prefix: &str) -> Result<StorageMap<K>, MetadataError> {
		match &self.ty {
			StorageEntryType::Map { hashers, .. } => {
				let hasher = hashers.first().ok_or(MetadataError::StorageTypeError)?.to_owned();

				let module_prefix = pallet_prefix.as_bytes().to_vec();
				let storage_prefix = self.name.as_bytes().to_vec();

				// hashers do not implement debug in no_std
				#[cfg(feature = "std")]
				log::debug!("map for '{}' '{}' has hasher {:?}", pallet_prefix, self.name, hasher);

				Ok(StorageMap { _marker: PhantomData, module_prefix, storage_prefix, hasher })
			},
			_ => Err(MetadataError::StorageTypeError),
		}
	}
	fn get_map_prefix(&self, pallet_prefix: &str) -> Result<StorageKey, MetadataError> {
		match &self.ty {
			StorageEntryType::Map { .. } => {
				let mut bytes = sp_crypto_hashing::twox_128(pallet_prefix.as_bytes()).to_vec();
				bytes.extend(&sp_crypto_hashing::twox_128(self.name.as_bytes())[..]);
				Ok(StorageKey(bytes))
			},
			_ => Err(MetadataError::StorageTypeError),
		}
	}

	fn get_double_map_prefix<K: Encode>(
		&self,
		pallet_prefix: &str,
		key1: K,
	) -> Result<StorageKey, MetadataError> {
		match &self.ty {
			StorageEntryType::Map { hashers, .. } => {
				let module_prefix = pallet_prefix.as_bytes();
				let storage_prefix = self.name.as_bytes();

				let hasher1 = hashers.first().ok_or(MetadataError::StorageTypeError)?;

				let mut bytes = sp_crypto_hashing::twox_128(module_prefix).to_vec();
				bytes.extend(&sp_crypto_hashing::twox_128(storage_prefix)[..]);
				bytes.extend(key_hash(&key1, hasher1));

				Ok(StorageKey(bytes))
			},
			_ => Err(MetadataError::StorageTypeError),
		}
	}

	fn get_value(&self, pallet_prefix: &str) -> Result<StorageValue, MetadataError> {
		match &self.ty {
			StorageEntryType::Plain { .. } => {
				let module_prefix = pallet_prefix.as_bytes().to_vec();
				let storage_prefix = self.name.as_bytes().to_vec();
				Ok(StorageValue { module_prefix, storage_prefix })
			},
			_ => Err(MetadataError::StorageTypeError),
		}
	}
}

/// generates the key's hash depending on the StorageHasher selected
fn key_hash<K: Encode>(key: &K, hasher: &StorageHasher) -> Vec<u8> {
	let encoded_key = key.encode();
	match hasher {
		StorageHasher::Identity => encoded_key.to_vec(),
		StorageHasher::Blake2_128 => sp_crypto_hashing::blake2_128(&encoded_key).to_vec(),
		StorageHasher::Blake2_128Concat => {
			// copied from substrate Blake2_128Concat::hash since StorageHasher is not public
			let x: &[u8] = encoded_key.as_slice();
			sp_crypto_hashing::blake2_128(x)
				.iter()
				.chain(x.iter())
				.cloned()
				.collect::<Vec<_>>()
		},
		StorageHasher::Blake2_256 => sp_crypto_hashing::blake2_256(&encoded_key).to_vec(),
		StorageHasher::Twox128 => sp_crypto_hashing::twox_128(&encoded_key).to_vec(),
		StorageHasher::Twox256 => sp_crypto_hashing::twox_256(&encoded_key).to_vec(),
		StorageHasher::Twox64Concat => sp_crypto_hashing::twox_64(&encoded_key)
			.iter()
			.chain(&encoded_key)
			.cloned()
			.collect(),
	}
}
