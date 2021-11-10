// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of subxt.
//
// subxt is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// subxt is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with subxt.  If not, see <http://www.gnu.org/licenses/>.

//! For querying runtime storage.

use codec::{Decode, Encode};
use log::info;
use sp_core::storage::StorageKey;
use std::marker::PhantomData;

use crate::metadata::MetadataError;
use frame_metadata::{StorageEntryMetadata, StorageEntryType, StorageHasher};
use scale_info::form::PortableForm;

#[derive(Clone, Debug)]
pub struct StorageValue {
    module_prefix: Vec<u8>,
    storage_prefix: Vec<u8>,
}

impl StorageValue {
    pub fn key(&self) -> StorageKey {
        let mut bytes = sp_core::twox_128(&self.module_prefix).to_vec();
        bytes.extend(&sp_core::twox_128(&self.storage_prefix)[..]);
        StorageKey(bytes)
    }
}

#[derive(Clone, Debug)]
pub struct StorageMap<K, V> {
    _marker: PhantomData<K>,
    module_prefix: Vec<u8>,
    storage_prefix: Vec<u8>,
    hasher: StorageHasher,
    default: V,
}

impl<K: Encode, V: Decode + Clone> StorageMap<K, V> {
    pub fn key(&self, key: K) -> StorageKey {
        let mut bytes = sp_core::twox_128(&self.module_prefix).to_vec();
        bytes.extend(&sp_core::twox_128(&self.storage_prefix)[..]);
        bytes.extend(key_hash(&key, &self.hasher));
        StorageKey(bytes)
    }

    pub fn default(&self) -> V {
        self.default.clone()
    }
}

#[derive(Clone, Debug)]
pub struct StorageDoubleMap<K, Q, V> {
    _marker: PhantomData<K>,
    _marker2: PhantomData<Q>,
    module_prefix: Vec<u8>,
    storage_prefix: Vec<u8>,
    hasher: StorageHasher,
    key2_hasher: StorageHasher,
    default: V,
}

impl<K: Encode, Q: Encode, V: Decode + Clone> StorageDoubleMap<K, Q, V> {
    pub fn key(&self, key1: K, key2: Q) -> StorageKey {
        let mut bytes = sp_core::twox_128(&self.module_prefix).to_vec();
        bytes.extend(&sp_core::twox_128(&self.storage_prefix)[..]);
        bytes.extend(key_hash(&key1, &self.hasher));
        bytes.extend(key_hash(&key2, &self.key2_hasher));
        StorageKey(bytes)
    }

    pub fn default(&self) -> V {
        self.default.clone()
    }
}

pub trait GetStorage {
    fn get_double_map<K: Encode, Q: Encode, V: Decode + Clone>(
        &self,
        pallet_prefix: &str,
    ) -> Result<StorageDoubleMap<K, Q, V>, MetadataError>;
    fn get_map<K: Encode, V: Decode + Clone>(
        &self,
        pallet_prefix: &str,
    ) -> Result<StorageMap<K, V>, MetadataError>;
    fn get_map_prefix(&self, pallet_prefix: &str) -> Result<StorageKey, MetadataError>;
    fn get_value(&self, pallet_prefix: &str) -> Result<StorageValue, MetadataError>;
}

impl GetStorage for StorageEntryMetadata<PortableForm> {
    fn get_double_map<K: Encode, Q: Encode, V: Decode + Clone>(
        &self,
        pallet_prefix: &str,
    ) -> Result<StorageDoubleMap<K, Q, V>, MetadataError> {
        match &self.ty {
            StorageEntryType::Map { hashers, .. } => {
                let module_prefix = pallet_prefix.as_bytes().to_vec();
                let storage_prefix = self.name.as_bytes().to_vec();
                let hasher1 = hashers.get(0).ok_or(MetadataError::StorageTypeError)?;
                let hasher2 = hashers.get(1).ok_or(MetadataError::StorageTypeError)?;

                let default = Decode::decode(&mut &self.default[..])
                    .map_err(|_| MetadataError::MapValueTypeError)?;

                info!(
                    "map for '{}' '{}' has hasher1 {:?} hasher2 {:?}",
                    pallet_prefix, self.name, hasher1, hasher2
                );
                Ok(StorageDoubleMap {
                    _marker: PhantomData,
                    _marker2: PhantomData,
                    module_prefix,
                    storage_prefix,
                    hasher: hasher1.to_owned(),
                    key2_hasher: hasher2.to_owned(),
                    default,
                })
            }
            _ => Err(MetadataError::StorageTypeError),
        }
    }
    fn get_map<K: Encode, V: Decode + Clone>(
        &self,
        pallet_prefix: &str,
    ) -> Result<StorageMap<K, V>, MetadataError> {
        match &self.ty {
            StorageEntryType::Map { hashers, .. } => {
                let hasher = hashers
                    .get(0)
                    .ok_or(MetadataError::StorageTypeError)?
                    .to_owned();

                let module_prefix = pallet_prefix.as_bytes().to_vec();
                let storage_prefix = self.name.as_bytes().to_vec();
                let default = Decode::decode(&mut &self.default[..])
                    .map_err(|_| MetadataError::MapValueTypeError)?;

                info!(
                    "map for '{}' '{}' has hasher {:?}",
                    pallet_prefix, self.name, hasher
                );
                Ok(StorageMap {
                    _marker: PhantomData,
                    module_prefix,
                    storage_prefix,
                    hasher,
                    default,
                })
            }
            _ => Err(MetadataError::StorageTypeError),
        }
    }
    fn get_map_prefix(&self, pallet_prefix: &str) -> Result<StorageKey, MetadataError> {
        match &self.ty {
            StorageEntryType::Map { .. } => {
                let mut bytes = sp_core::twox_128(&pallet_prefix.as_bytes().to_vec()).to_vec();
                bytes.extend(&sp_core::twox_128(&self.name.as_bytes().to_vec())[..]);
                Ok(StorageKey(bytes))
            }
            _ => Err(MetadataError::StorageTypeError),
        }
    }

    fn get_value(&self, pallet_prefix: &str) -> Result<StorageValue, MetadataError> {
        match &self.ty {
            StorageEntryType::Plain { .. } => {
                let module_prefix = pallet_prefix.as_bytes().to_vec();
                let storage_prefix = self.name.as_bytes().to_vec();
                Ok(StorageValue {
                    module_prefix,
                    storage_prefix,
                })
            }
            _ => Err(MetadataError::StorageTypeError),
        }
    }
}

/// generates the key's hash depending on the StorageHasher selected
fn key_hash<K: Encode>(key: &K, hasher: &StorageHasher) -> Vec<u8> {
    let encoded_key = key.encode();
    match hasher {
        StorageHasher::Identity => encoded_key.to_vec(),
        StorageHasher::Blake2_128 => sp_core::blake2_128(&encoded_key).to_vec(),
        StorageHasher::Blake2_128Concat => {
            // copied from substrate Blake2_128Concat::hash since StorageHasher is not public
            let x: &[u8] = encoded_key.as_slice();
            sp_core::blake2_128(x)
                .iter()
                .chain(x.iter())
                .cloned()
                .collect::<Vec<_>>()
        }
        StorageHasher::Blake2_256 => sp_core::blake2_256(&encoded_key).to_vec(),
        StorageHasher::Twox128 => sp_core::twox_128(&encoded_key).to_vec(),
        StorageHasher::Twox256 => sp_core::twox_256(&encoded_key).to_vec(),
        StorageHasher::Twox64Concat => sp_core::twox_64(&encoded_key)
            .iter()
            .chain(&encoded_key)
            .cloned()
            .collect(),
    }
}
