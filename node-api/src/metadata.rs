// Copyright 2019-2021 Parity Technologies (UK) Ltd. and Supercomputing Systems AG
// and Integritee AG.
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

//! Handle substrate chain metadata
//!
//! This file is mostly subxt.

use crate::{storage::GetStorage, Encoded};
use codec::{Decode, Encode, Error as CodecError};
use frame_metadata::{
    PalletConstantMetadata, RuntimeMetadata, RuntimeMetadataLastVersion, RuntimeMetadataPrefixed,
    StorageEntryMetadata, META_RESERVED,
};
use scale_info::{form::PortableForm, Type, Variant};
use sp_core::storage::StorageKey;

#[cfg(feature = "std")]
use serde::Serialize;

// We use `BTreeMap` because we can't use `HashMap` in `no_std`.
use sp_std::collections::btree_map::BTreeMap;

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

/// Metadata error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataError {
    /// Module is not in metadata.
    PalletNotFound(String),
    /// Pallet is not in metadata.
    PalletIndexNotFound(u8),
    /// Call is not in metadata.
    CallNotFound(&'static str),
    /// Event is not in metadata.
    EventNotFound(u8, u8),
    /// Error is not in metadata.
    ErrorNotFound(u8, u8),
    /// Storage is not in metadata.
    StorageNotFound(&'static str),
    /// Storage type does not match requested type.
    StorageTypeError,
    /// Map value type does not match requested type.
    MapValueTypeError,
    /// Failed to decode the value's default.
    DefaultError(CodecError),
    /// Failure to decode constant value.
    ConstantValueError(CodecError),
    /// Constant is not in metadata.
    ConstantNotFound(&'static str),
    /// Type is missing from type registry.
    TypeNotFound(u32),
}

/// Runtime metadata.
#[derive(Clone, Debug, Encode, Decode)]
pub struct Metadata {
    pub metadata: RuntimeMetadataLastVersion,
    pub pallets: BTreeMap<String, PalletMetadata>,
    pub events: BTreeMap<(u8, u8), EventMetadata>,
    pub errors: BTreeMap<(u8, u8), ErrorMetadata>,
}

impl Metadata {
    /// Returns a reference to [`PalletMetadata`].
    pub fn pallet(&self, name: &'static str) -> Result<&PalletMetadata, MetadataError> {
        self.pallets
            .get(name)
            .ok_or_else(|| MetadataError::PalletNotFound(name.to_string()))
    }

    /// Returns the metadata for the event at the given pallet and event indices.
    pub fn event(
        &self,
        pallet_index: u8,
        event_index: u8,
    ) -> Result<&EventMetadata, MetadataError> {
        let event = self
            .events
            .get(&(pallet_index, event_index))
            .ok_or(MetadataError::EventNotFound(pallet_index, event_index))?;
        Ok(event)
    }

    /// Returns the metadata for all events of a given pallet
    pub fn events(&self, pallet_index: u8) -> Vec<EventMetadata> {
        self.events
            .clone()
            .into_iter()
            .filter(|(k, _v)| k.0 == pallet_index)
            .map(|(_k, v)| v)
            .collect()
    }

    /// Returns the metadata for the error at the given pallet and error indices.
    pub fn error(
        &self,
        pallet_index: u8,
        error_index: u8,
    ) -> Result<&ErrorMetadata, MetadataError> {
        let error = self
            .errors
            .get(&(pallet_index, error_index))
            .ok_or(MetadataError::ErrorNotFound(pallet_index, error_index))?;
        Ok(error)
    }

    /// Returns the metadata for all errors of a given pallet
    pub fn errors(&self, pallet_index: u8) -> Vec<ErrorMetadata> {
        self.errors
            .clone()
            .into_iter()
            .filter(|(k, _v)| k.0 == pallet_index)
            .map(|(_k, v)| v)
            .collect()
    }

    /// Resolve a type definition.
    pub fn resolve_type(&self, id: u32) -> Option<&Type<PortableForm>> {
        self.metadata.types.resolve(id)
    }

    /// Return the runtime metadata.
    pub fn runtime_metadata(&self) -> &RuntimeMetadataLastVersion {
        &self.metadata
    }

    #[cfg(feature = "std")]
    pub fn pretty_format(metadata: &RuntimeMetadataPrefixed) -> Option<String> {
        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b" ");
        let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
        metadata.serialize(&mut ser).unwrap();
        String::from_utf8(ser.into_inner()).ok()
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct PalletMetadata {
    pub index: u8,
    pub name: String,
    pub calls: BTreeMap<String, u8>,
    pub storage: BTreeMap<String, StorageEntryMetadata<PortableForm>>,
    pub constants: BTreeMap<String, PalletConstantMetadata<PortableForm>>,
}

impl PalletMetadata {
    pub fn encode_call<C>(&self, call_name: &'static str, args: C) -> Result<Encoded, MetadataError>
    where
        C: Encode,
    {
        let fn_index = self
            .calls
            .get(call_name)
            .ok_or(MetadataError::CallNotFound(call_name))?;
        let mut bytes = vec![self.index, *fn_index];
        bytes.extend(args.encode());
        Ok(Encoded(bytes))
    }

    pub fn storage(
        &self,
        key: &'static str,
    ) -> Result<&StorageEntryMetadata<PortableForm>, MetadataError> {
        self.storage
            .get(key)
            .ok_or(MetadataError::StorageNotFound(key))
    }

    /// Get a constant's metadata by name
    pub fn constant(
        &self,
        key: &'static str,
    ) -> Result<&PalletConstantMetadata<PortableForm>, MetadataError> {
        self.constants
            .get(key)
            .ok_or(MetadataError::ConstantNotFound(key))
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct EventMetadata {
    pub pallet: String,
    pub event: String,
    pub variant: Variant<PortableForm>,
}

impl EventMetadata {
    /// Get the name of the pallet from which the event was emitted.
    pub fn pallet(&self) -> &str {
        &self.pallet
    }

    /// Get the name of the pallet event which was emitted.
    pub fn event(&self) -> &str {
        &self.event
    }

    /// Get the type def variant for the pallet event.
    pub fn variant(&self) -> &Variant<PortableForm> {
        &self.variant
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Encode, Decode)]
pub struct ErrorMetadata {
    pub pallet: String,
    pub error: String,
    pub variant: Variant<PortableForm>,
}

impl ErrorMetadata {
    /// Get the name of the pallet from which the error originates.
    pub fn pallet(&self) -> &str {
        &self.pallet
    }

    /// Get the name of the specific pallet error.
    pub fn error(&self) -> &str {
        &self.error
    }

    /// Get the description of the specific pallet error.
    pub fn description(&self) -> &[String] {
        self.variant.docs()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Encode, Decode)]
pub enum InvalidMetadataError {
    InvalidPrefix,
    InvalidVersion,
    /// Type is missing from type registry.
    MissingType(u32),
    /// Type was not variant/enum type.
    TypeDefNotVariant(u32),
}

impl TryFrom<RuntimeMetadataPrefixed> for Metadata {
    type Error = InvalidMetadataError;

    fn try_from(metadata: RuntimeMetadataPrefixed) -> Result<Self, Self::Error> {
        if metadata.0 != META_RESERVED {
            return Err(InvalidMetadataError::InvalidPrefix);
        }
        let metadata = match metadata.1 {
            RuntimeMetadata::V14(meta) => meta,
            _ => return Err(InvalidMetadataError::InvalidVersion),
        };

        let get_type_def_variant = |type_id: u32| {
            let ty = metadata
                .types
                .resolve(type_id)
                .ok_or(InvalidMetadataError::MissingType(type_id))?;
            if let scale_info::TypeDef::Variant(var) = ty.type_def() {
                Ok(var)
            } else {
                Err(InvalidMetadataError::TypeDefNotVariant(type_id))
            }
        };
        let pallets = metadata
            .pallets
            .iter()
            .map(|pallet| {
                let calls = pallet.calls.as_ref().map_or(Ok(BTreeMap::new()), |call| {
                    let type_def_variant = get_type_def_variant(call.ty.id())?;
                    let calls = type_def_variant
                        .variants()
                        .iter()
                        .map(|v| (v.name().clone(), v.index()))
                        .collect();
                    Ok(calls)
                })?;

                let storage = pallet.storage.as_ref().map_or(BTreeMap::new(), |storage| {
                    storage
                        .entries
                        .iter()
                        .map(|entry| (entry.name.clone(), entry.clone()))
                        .collect()
                });

                let constants = pallet
                    .constants
                    .iter()
                    .map(|constant| (constant.name.clone(), constant.clone()))
                    .collect();

                let pallet_metadata = PalletMetadata {
                    index: pallet.index,
                    name: pallet.name.to_string(),
                    calls,
                    storage,
                    constants,
                };

                Ok((pallet.name.to_string(), pallet_metadata))
            })
            .collect::<Result<_, _>>()?;

        let pallet_events = metadata
            .pallets
            .iter()
            .filter_map(|pallet| {
                pallet.event.as_ref().map(|event| {
                    let type_def_variant = get_type_def_variant(event.ty.id())?;
                    Ok((pallet, type_def_variant))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let events = pallet_events
            .iter()
            .flat_map(|(pallet, type_def_variant)| {
                type_def_variant.variants().iter().map(move |var| {
                    let key = (pallet.index, var.index());
                    let value = EventMetadata {
                        pallet: pallet.name.clone(),
                        event: var.name().clone(),
                        variant: var.clone(),
                    };
                    (key, value)
                })
            })
            .collect();

        let pallet_errors = metadata
            .pallets
            .iter()
            .filter_map(|pallet| {
                pallet.error.as_ref().map(|error| {
                    let type_def_variant = get_type_def_variant(error.ty.id())?;
                    Ok((pallet, type_def_variant))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let errors = pallet_errors
            .iter()
            .flat_map(|(pallet, type_def_variant)| {
                type_def_variant.variants().iter().map(move |var| {
                    let key = (pallet.index, var.index());
                    let value = ErrorMetadata {
                        pallet: pallet.name.clone(),
                        error: var.name().clone(),
                        variant: var.clone(),
                    };
                    (key, value)
                })
            })
            .collect();

        Ok(Self {
            metadata,
            pallets,
            events,
            errors,
        })
    }
}

/// Get the storage keys corresponding to a storage
///
/// This is **not** part of subxt.
impl Metadata {
    pub fn storage_value_key(
        &self,
        storage_prefix: &'static str,
        storage_key_name: &'static str,
    ) -> Result<StorageKey, MetadataError> {
        Ok(self
            .pallet(storage_prefix)?
            .storage(storage_key_name)?
            .get_value(storage_prefix)?
            .key())
    }

    pub fn storage_map_key<K: Encode>(
        &self,
        storage_prefix: &'static str,
        storage_key_name: &'static str,
        map_key: K,
    ) -> Result<StorageKey, MetadataError> {
        Ok(self
            .pallet(storage_prefix)?
            .storage(storage_key_name)?
            .get_map::<K>(storage_prefix)?
            .key(map_key))
    }

    pub fn storage_map_key_prefix(
        &self,
        storage_prefix: &'static str,
        storage_key_name: &'static str,
    ) -> Result<StorageKey, MetadataError> {
        self.pallet(storage_prefix)?
            .storage(storage_key_name)?
            .get_map_prefix(storage_prefix)
    }

    pub fn storage_double_map_key<K: Encode, Q: Encode>(
        &self,
        storage_prefix: &'static str,
        storage_key_name: &'static str,
        first: K,
        second: Q,
    ) -> Result<StorageKey, MetadataError> {
        Ok(self
            .pallet(storage_prefix)?
            .storage(storage_key_name)?
            .get_double_map::<K, Q>(storage_prefix)?
            .key(first, second))
    }
}
