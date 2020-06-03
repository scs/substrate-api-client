// Copyright 2019 Parity Technologies (UK) Ltd. and Supercomputing Systems AG
// This file is part of substrate-subxt.
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
// along with substrate-subxt.  If not, see <http://www.gnu.org/licenses/>.

use std::{collections::HashMap, convert::TryFrom, marker::PhantomData, str::FromStr};

use codec::{Decode, Encode};

use log::*;
use metadata::{
    DecodeDifferent, RuntimeMetadata, RuntimeMetadataPrefixed, StorageEntryModifier,
    StorageEntryType, StorageHasher, META_RESERVED,
};
use serde::ser::Serialize;
use sp_core::storage::StorageKey;

#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("Error converting substrate metadata: {0}")]
    Conversion(#[from] ConversionError),
    #[error("Module not found")]
    ModuleNotFound(String),
    #[error("Module with events not found")]
    ModuleWithEventsNotFound(u8),
    #[error("Call not found")]
    CallNotFound(&'static str),
    #[error("Event not found")]
    EventNotFound(u8),
    #[error("Storage not found")]
    StorageNotFound(&'static str),
    #[error("Storage type error")]
    StorageTypeError,
    #[error("Map value type error")]
    MapValueTypeError,
}

#[derive(Clone, Debug)]
pub struct Metadata {
    modules: HashMap<String, ModuleMetadata>,
    modules_with_calls: HashMap<String, ModuleWithCalls>,
    modules_with_events: HashMap<String, ModuleWithEvents>,
}

impl Metadata {
    pub fn module<S>(&self, name: S) -> Result<&ModuleMetadata, MetadataError>
    where
        S: ToString,
    {
        let name = name.to_string();
        self.modules
            .get(&name)
            .ok_or(MetadataError::ModuleNotFound(name))
    }

    pub fn modules_with_calls(&self) -> impl Iterator<Item = &ModuleWithCalls> {
        self.modules_with_calls.values()
    }

    pub fn module_with_calls<S>(&self, name: S) -> Result<&ModuleWithCalls, MetadataError>
    where
        S: ToString,
    {
        let name = name.to_string();
        self.modules_with_calls
            .get(&name)
            .ok_or(MetadataError::ModuleNotFound(name))
    }

    pub fn modules_with_events(&self) -> impl Iterator<Item = &ModuleWithEvents> {
        self.modules_with_events.values()
    }

    pub fn module_with_events_by_name<S>(&self, name: S) -> Result<&ModuleWithEvents, MetadataError>
    where
        S: ToString,
    {
        let name = name.to_string();
        self.modules_with_events
            .get(&name)
            .ok_or(MetadataError::ModuleNotFound(name))
    }

    pub fn module_with_events(&self, module_index: u8) -> Result<&ModuleWithEvents, MetadataError> {
        self.modules_with_events
            .values()
            .find(|&module| module.index == module_index)
            .ok_or(MetadataError::ModuleWithEventsNotFound(module_index))
    }

    pub fn print_overview(&self) {
        let mut string = String::new();
        for (name, module) in &self.modules {
            string.push_str(name.as_str());
            string.push('\n');
            for storage in module.storage.keys() {
                string.push_str(" s  ");
                string.push_str(storage.as_str());
                string.push('\n');
            }
            if let Some(module) = self.modules_with_calls.get(name) {
                for call in module.calls.keys() {
                    string.push_str(" c  ");
                    string.push_str(call.as_str());
                    string.push('\n');
                }
            }
            if let Some(module) = self.modules_with_events.get(name) {
                for event in module.events.values() {
                    string.push_str(" e  ");
                    string.push_str(event.name.as_str());
                    string.push('\n');
                }
            }
        }
        println!("{}", string);
    }

    pub fn pretty_format(metadata: &RuntimeMetadataPrefixed) -> Option<String> {
        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b" ");
        let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
        metadata.serialize(&mut ser).unwrap();
        String::from_utf8(ser.into_inner()).ok()
    }

    pub fn print_modules_with_calls(&self) {
        for m in self.modules_with_calls() {
            m.print()
        }
    }

    pub fn print_modules_with_events(&self) {
        for m in self.modules_with_events() {
            m.print()
        }
    }

    pub fn storage_value_key(
        &self,
        storage_prefix: &'static str,
        storage_key_name: &'static str,
    ) -> Result<StorageKey, MetadataError> {
        Ok(self
            .module(storage_prefix)?
            .storage(storage_key_name)?
            .get_value()?
            .key())
    }

    pub fn storage_map_key<K: Encode, V: Decode + Clone>(
        &self,
        storage_prefix: &'static str,
        storage_key_name: &'static str,
        map_key: K,
    ) -> Result<StorageKey, MetadataError> {
        Ok(self
            .module(storage_prefix)?
            .storage(storage_key_name)?
            .get_map::<K, V>()?
            .key(map_key))
    }

    pub fn storage_map_key_prefix(
        &self,
        storage_prefix: &'static str,
        storage_key_name: &'static str,
    ) -> Result<StorageKey, MetadataError> {
        self.module(storage_prefix)?
            .storage(storage_key_name)?
            .get_map_prefix()
    }

    pub fn storage_double_map_key<K: Encode, Q: Encode, V: Decode + Clone>(
        &self,
        storage_prefix: &'static str,
        storage_key_name: &'static str,
        first: K,
        second: Q,
    ) -> Result<StorageKey, MetadataError> {
        Ok(self
            .module(storage_prefix)?
            .storage(storage_key_name)?
            .get_double_map::<K, Q, V>()?
            .key(first, second))
    }
}

#[derive(Clone, Debug)]
pub struct ModuleMetadata {
    name: String,
    storage: HashMap<String, StorageMetadata>,
    // constants
}

impl ModuleMetadata {
    pub fn storage(&self, key: &'static str) -> Result<&StorageMetadata, MetadataError> {
        self.storage
            .get(key)
            .ok_or(MetadataError::StorageNotFound(key))
    }
}

// Todo make nice list of Call args to facilitate call arg lookup
#[derive(Clone, Debug)]
pub struct ModuleWithCalls {
    pub index: u8,
    pub name: String,
    pub calls: HashMap<String, u8>,
}

impl ModuleWithCalls {
    pub fn print(&self) {
        println!(
            "----------------- Calls for Module: '{}' -----------------\n",
            self.name
        );
        for (name, index) in &self.calls {
            println!("Name: {}, index {}", name, index);
        }
        println!()
    }
}

#[derive(Clone, Debug)]
pub struct ModuleWithEvents {
    index: u8,
    name: String,
    events: HashMap<u8, ModuleEventMetadata>,
}

impl ModuleWithEvents {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn events(&self) -> impl Iterator<Item = &ModuleEventMetadata> {
        self.events.values()
    }

    pub fn event(&self, index: u8) -> Result<&ModuleEventMetadata, MetadataError> {
        self.events
            .get(&index)
            .ok_or(MetadataError::EventNotFound(index))
    }

    pub fn print(&self) {
        println!(
            "----------------- Events for Module: {} -----------------\n",
            self.name()
        );

        for e in self.events() {
            println!("Name: {:?}, Args: {:?}", e.name, e.arguments);
        }
        println!()
    }
}

#[derive(Clone, Debug)]
pub struct StorageMetadata {
    module_prefix: String,
    storage_prefix: String,
    modifier: StorageEntryModifier,
    ty: StorageEntryType,
    default: Vec<u8>,
}

impl StorageMetadata {
    pub fn get_double_map<K: Encode, Q: Encode, V: Decode + Clone>(
        &self,
    ) -> Result<StorageDoubleMap<K, Q, V>, MetadataError> {
        match &self.ty {
            StorageEntryType::DoubleMap {
                hasher,
                key2_hasher,
                ..
            } => {
                let module_prefix = self.module_prefix.as_bytes().to_vec();
                let storage_prefix = self.storage_prefix.as_bytes().to_vec();
                let hasher1 = hasher.to_owned();
                let hasher2 = key2_hasher.to_owned();

                let default = Decode::decode(&mut &self.default[..])
                    .map_err(|_| MetadataError::MapValueTypeError)?;

                info!(
                    "map for '{}' '{}' has hasher1 {:?} hasher2 {:?}",
                    self.module_prefix, self.storage_prefix, hasher1, hasher2
                );
                Ok(StorageDoubleMap {
                    _marker: PhantomData,
                    _marker2: PhantomData,
                    module_prefix,
                    storage_prefix,
                    hasher: hasher1,
                    key2_hasher: hasher2,
                    default,
                })
            }
            _ => Err(MetadataError::StorageTypeError),
        }
    }
    pub fn get_map<K: Encode, V: Decode + Clone>(&self) -> Result<StorageMap<K, V>, MetadataError> {
        match &self.ty {
            StorageEntryType::Map { hasher, .. } => {
                let module_prefix = self.module_prefix.as_bytes().to_vec();
                let storage_prefix = self.storage_prefix.as_bytes().to_vec();
                let hasher = hasher.to_owned();
                let default = Decode::decode(&mut &self.default[..])
                    .map_err(|_| MetadataError::MapValueTypeError)?;

                info!(
                    "map for '{}' '{}' has hasher {:?}",
                    self.module_prefix, self.storage_prefix, hasher
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
    pub fn get_map_prefix(&self) -> Result<StorageKey, MetadataError> {
        match &self.ty {
            StorageEntryType::Map { .. } => {
                let mut bytes = sp_core::twox_128(&self.module_prefix.as_bytes().to_vec()).to_vec();
                bytes.extend(&sp_core::twox_128(&self.storage_prefix.as_bytes().to_vec())[..]);
                Ok(StorageKey(bytes))
            }
            _ => Err(MetadataError::StorageTypeError),
        }
    }

    pub fn get_value(&self) -> Result<StorageValue, MetadataError> {
        match &self.ty {
            StorageEntryType::Plain { .. } => {
                let module_prefix = self.module_prefix.as_bytes().to_vec();
                let storage_prefix = self.storage_prefix.as_bytes().to_vec();
                Ok(StorageValue {
                    module_prefix,
                    storage_prefix,
                })
            }
            _ => Err(MetadataError::StorageTypeError),
        }
    }
}

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

#[derive(Clone, Debug)]
pub struct ModuleEventMetadata {
    pub name: String,
    arguments: Vec<EventArg>,
}

impl ModuleEventMetadata {
    pub fn arguments(&self) -> Vec<EventArg> {
        self.arguments.to_vec()
    }
}

/// Naive representation of event argument types, supports current set of substrate EventArg types.
/// If and when Substrate uses `type-metadata`, this can be replaced.
///
/// Used to calculate the size of a instance of an event variant without having the concrete type,
/// so the raw bytes can be extracted from the encoded `Vec<EventRecord<E>>` (without `E` defined).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum EventArg {
    Primitive(String),
    Vec(Box<EventArg>),
    Tuple(Vec<EventArg>),
}

impl FromStr for EventArg {
    type Err = ConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("Vec<") {
            if s.ends_with('>') {
                Ok(EventArg::Vec(Box::new(s[4..s.len() - 1].parse()?)))
            } else {
                Err(ConversionError::InvalidEventArg(
                    s.to_string(),
                    "Expected closing `>` for `Vec`",
                ))
            }
        } else if s.starts_with('(') {
            if s.ends_with(')') {
                let mut args = Vec::new();
                for arg in s[1..s.len() - 1].split(',') {
                    let arg = arg.trim().parse()?;
                    args.push(arg)
                }
                Ok(EventArg::Tuple(args))
            } else {
                Err(ConversionError::InvalidEventArg(
                    s.to_string(),
                    "Expecting closing `)` for tuple",
                ))
            }
        } else {
            Ok(EventArg::Primitive(s.to_string()))
        }
    }
}

impl EventArg {
    /// Returns all primitive types for this EventArg
    pub fn primitives(&self) -> Vec<String> {
        match self {
            EventArg::Primitive(p) => vec![p.clone()],
            EventArg::Vec(arg) => arg.primitives(),
            EventArg::Tuple(args) => {
                let mut primitives = Vec::new();
                for arg in args {
                    primitives.extend(arg.primitives())
                }
                primitives
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Invalid prefix")]
    InvalidPrefix,
    #[error("Invalid version")]
    InvalidVersion,
    #[error("Expected DecodeDifferent::Decoded")]
    ExpectedDecoded,
    #[error("Invalid event arg {0}")]
    InvalidEventArg(String, &'static str),
}

impl TryFrom<RuntimeMetadataPrefixed> for Metadata {
    type Error = MetadataError;

    fn try_from(metadata: RuntimeMetadataPrefixed) -> Result<Self, Self::Error> {
        if metadata.0 != META_RESERVED {
            return Err(ConversionError::InvalidPrefix.into());
        }
        let meta = match metadata.1 {
            RuntimeMetadata::V11(meta) => meta,
            _ => return Err(ConversionError::InvalidVersion.into()),
        };
        let mut modules = HashMap::new();
        let mut modules_with_calls = HashMap::new();
        let mut modules_with_events = HashMap::new();
        for module in convert(meta.modules)?.into_iter() {
            let module_name = convert(module.name.clone())?;

            let mut storage_map = HashMap::new();
            if let Some(storage) = module.storage {
                let storage = convert(storage)?;
                let module_prefix = convert(storage.prefix)?;
                for entry in convert(storage.entries)?.into_iter() {
                    let storage_prefix = convert(entry.name.clone())?;
                    let entry =
                        convert_entry(module_prefix.clone(), storage_prefix.clone(), entry)?;
                    storage_map.insert(storage_prefix, entry);
                }
            }
            modules.insert(
                module_name.clone(),
                ModuleMetadata {
                    name: module_name.clone(),
                    storage: storage_map,
                },
            );

            if let Some(calls) = module.calls {
                let mut call_map = HashMap::new();
                for (index, call) in convert(calls)?.into_iter().enumerate() {
                    let name = convert(call.name)?;
                    call_map.insert(name, index as u8);
                }
                modules_with_calls.insert(
                    module_name.clone(),
                    ModuleWithCalls {
                        index: modules_with_calls.len() as u8,
                        name: module_name.clone(),
                        calls: call_map,
                    },
                );
            }
            if let Some(events) = module.event {
                let mut event_map = HashMap::new();
                for (index, event) in convert(events)?.into_iter().enumerate() {
                    event_map.insert(index as u8, convert_event(event)?);
                }
                modules_with_events.insert(
                    module_name.clone(),
                    ModuleWithEvents {
                        index: modules_with_events.len() as u8,
                        name: module_name.clone(),
                        events: event_map,
                    },
                );
            }
        }
        Ok(Metadata {
            modules,
            modules_with_calls,
            modules_with_events,
        })
    }
}

fn convert<B: 'static, O: 'static>(dd: DecodeDifferent<B, O>) -> Result<O, ConversionError> {
    match dd {
        DecodeDifferent::Decoded(value) => Ok(value),
        _ => Err(ConversionError::ExpectedDecoded),
    }
}

fn convert_event(event: metadata::EventMetadata) -> Result<ModuleEventMetadata, ConversionError> {
    let name = convert(event.name)?;
    let mut arguments = Vec::new();
    for arg in convert(event.arguments)? {
        let arg = arg.parse::<EventArg>()?;
        arguments.push(arg);
    }
    Ok(ModuleEventMetadata { name, arguments })
}

fn convert_entry(
    module_prefix: String,
    storage_prefix: String,
    entry: metadata::StorageEntryMetadata,
) -> Result<StorageMetadata, ConversionError> {
    let default = convert(entry.default)?;
    Ok(StorageMetadata {
        module_prefix,
        storage_prefix,
        modifier: entry.modifier,
        ty: entry.ty,
        default,
    })
}

/// genertes the key's hash depending on the StorageHasher selected
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
        StorageHasher::Twox64Concat => sp_core::twox_64(&encoded_key).to_vec(),
    }
}
