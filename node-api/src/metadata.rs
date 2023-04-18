// This file was taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! Handle substrate chain metadata
//!
//! This file is mostly subxt.

use crate::{alloc::borrow::ToOwned, storage::GetStorageTypes, Encoded};
use ac_primitives::StorageKey;
use codec::{Decode, Encode, Error as CodecError};
use frame_metadata::{
	PalletConstantMetadata, RuntimeMetadata, RuntimeMetadataLastVersion, RuntimeMetadataPrefixed,
	StorageEntryMetadata, META_RESERVED,
};
use scale_info::{form::PortableForm, PortableRegistry, Type};

#[cfg(feature = "std")]
use serde::Serialize;

use alloc::{
	// We use `BTreeMap` because we can't use `HashMap` in `no_std`.
	collections::btree_map::BTreeMap,
	string::{String, ToString},
	vec,
	vec::Vec,
};

/// Metadata error originated from inspecting the internal representation of the runtime metadata.
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
	/// Default error.
	DefaultError(CodecError),
	/// Failure to decode constant value.
	ConstantValueError(CodecError),
	/// Constant is not in metadata.
	ConstantNotFound(&'static str),
	/// Type is not in metadata.
	TypeNotFound(u32),
	/// Runtime constant metadata is incompatible with the static one.
	IncompatibleConstantMetadata(String, String),
	/// Runtime call metadata is incompatible with the static one.
	IncompatibleCallMetadata(String, String),
	/// Runtime storage metadata is incompatible with the static one.
	IncompatibleStorageMetadata(String, String),
	/// Runtime metadata is not fully compatible with the static one.
	IncompatibleMetadata,
}

/// Metadata wrapper around the runtime metadata. Offers some extra features,
/// such as direct pallets, events and error access.
#[derive(Clone, Debug, Encode, Decode)]
pub struct Metadata {
	pub runtime_metadata: RuntimeMetadataLastVersion,
	pub pallets: BTreeMap<String, PalletMetadata>,
	pub events: BTreeMap<(u8, u8), EventMetadata>,
	pub errors: BTreeMap<(u8, u8), ErrorMetadata>,
	// Type of the DispatchError type, which is what comes back if
	// an extrinsic fails.
	dispatch_error_ty: Option<u32>,
	// subxt implements caches, but this is not no_std compatible,
	// so we leave it commented for the time being.
	// Could be made available in std modus #307

	// cached_metadata_hash: RwLock<Option<[u8; 32]>>,
	// cached_call_hashes: HashCache,
	// cached_constant_hashes: HashCache,
	// cached_storage_hashes: HashCache,
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

	/// Returns the metadata for all events of a given pallet.
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

	/// Returns the metadata for all errors of a given pallet.
	pub fn errors(&self, pallet_index: u8) -> Vec<ErrorMetadata> {
		self.errors
			.clone()
			.into_iter()
			.filter(|(k, _v)| k.0 == pallet_index)
			.map(|(_k, v)| v)
			.collect()
	}

	/// Return the DispatchError type ID if it exists.
	pub fn dispatch_error_ty(&self) -> Option<u32> {
		self.dispatch_error_ty
	}

	/// Return the type registry embedded within the metadata.
	pub fn types(&self) -> &PortableRegistry {
		&self.runtime_metadata.types
	}

	/// Resolve a type definition.
	pub fn resolve_type(&self, id: u32) -> Option<&Type<PortableForm>> {
		self.runtime_metadata.types.resolve(id)
	}

	/// Return the runtime metadata.
	pub fn runtime_metadata(&self) -> &RuntimeMetadataLastVersion {
		&self.runtime_metadata
	}

	#[cfg(feature = "std")]
	pub fn pretty_format(&self) -> Result<String, std::string::FromUtf8Error> {
		let buf = Vec::new();
		let formatter = serde_json::ser::PrettyFormatter::with_indent(b" ");
		let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
		self.runtime_metadata.serialize(&mut ser).unwrap();
		String::from_utf8(ser.into_inner())
	}
}

/// Metadata for a specific pallet.
#[derive(Clone, Debug, Encode, Decode)]
pub struct PalletMetadata {
	pub index: u8,
	pub name: String,
	pub call_indexes: BTreeMap<String, u8>,
	pub call_ty_id: Option<u32>,
	pub storage: BTreeMap<String, StorageEntryMetadata<PortableForm>>,
	pub constants: BTreeMap<String, PalletConstantMetadata<PortableForm>>,
}

impl PalletMetadata {
	/// Get the name of the pallet.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get the index of this pallet.
	pub fn index(&self) -> u8 {
		self.index
	}

	/// If calls exist for this pallet, this returns the type ID of the variant
	/// representing the different possible calls.
	pub fn call_ty_id(&self) -> Option<u32> {
		self.call_ty_id
	}

	/// Attempt to resolve a call into an index in this pallet, failing
	/// if the call is not found in this pallet.
	pub fn call_index(&self, function: &'static str) -> Result<u8, MetadataError> {
		let fn_index =
			*self.call_indexes.get(function).ok_or(MetadataError::CallNotFound(function))?;
		Ok(fn_index)
	}

	pub fn storage(
		&self,
		key: &'static str,
	) -> Result<&StorageEntryMetadata<PortableForm>, MetadataError> {
		self.storage.get(key).ok_or(MetadataError::StorageNotFound(key))
	}

	/// Get a constant's metadata by name
	pub fn constant(
		&self,
		key: &'static str,
	) -> Result<&PalletConstantMetadata<PortableForm>, MetadataError> {
		self.constants.get(key).ok_or(MetadataError::ConstantNotFound(key))
	}

	pub fn encode_call<C>(&self, call_name: &'static str, args: C) -> Result<Encoded, MetadataError>
	where
		C: Encode,
	{
		let fn_index = self.call_index(call_name)?;
		let mut bytes = vec![self.index, fn_index];
		bytes.extend(args.encode());
		Ok(Encoded(bytes))
	}
}

/// Metadata for specific field.
#[derive(Clone, Debug, Encode, Decode)]
pub struct EventFieldMetadata {
	name: Option<String>,
	type_name: Option<String>,
	type_id: u32,
}

impl EventFieldMetadata {
	/// Construct a new [`EventFieldMetadata`]
	pub fn new(name: Option<String>, type_name: Option<String>, type_id: u32) -> Self {
		EventFieldMetadata { name, type_name, type_id }
	}

	/// Get the name of the field.
	pub fn name(&self) -> Option<&str> {
		self.name.as_deref()
	}

	/// Get the type name of the field as it appears in the code
	pub fn type_name(&self) -> Option<&str> {
		self.type_name.as_deref()
	}

	/// Get the id of a type
	pub fn type_id(&self) -> u32 {
		self.type_id
	}
}

/// Metadata for specific events.
#[derive(Clone, Debug, Encode, Decode)]
pub struct EventMetadata {
	pallet: String,
	event: String,
	fields: Vec<EventFieldMetadata>,
	docs: Vec<String>,
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

	/// The names, type names & types of each field in the event.
	pub fn fields(&self) -> &[EventFieldMetadata] {
		&self.fields
	}

	/// Documentation for this event.
	pub fn docs(&self) -> &[String] {
		&self.docs
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Encode, Decode)]
pub struct ErrorMetadata {
	pallet: String,
	error: String,
	docs: Vec<String>,
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

	/// Documentation for the error.
	pub fn docs(&self) -> &[String] {
		&self.docs
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
			return Err(InvalidMetadataError::InvalidPrefix)
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
			if let scale_info::TypeDef::Variant(var) = &ty.type_def {
				Ok(var)
			} else {
				Err(InvalidMetadataError::TypeDefNotVariant(type_id))
			}
		};
		let pallets = metadata
			.pallets
			.iter()
			.map(|pallet| {
				let call_ty_id = pallet.calls.as_ref().map(|c| c.ty.id);

				let call_indexes = pallet.calls.as_ref().map_or(Ok(BTreeMap::new()), |call| {
					let type_def_variant = get_type_def_variant(call.ty.id)?;
					let call_indexes = type_def_variant
						.variants
						.iter()
						.map(|v| (v.name.clone(), v.index))
						.collect();
					Ok(call_indexes)
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
					call_indexes,
					call_ty_id,
					storage,
					constants,
				};

				Ok((pallet.name.to_string(), pallet_metadata))
			})
			.collect::<Result<_, _>>()?;

		let mut events = BTreeMap::<(u8, u8), EventMetadata>::new();
		for pallet in &metadata.pallets {
			if let Some(event) = &pallet.event {
				let pallet_name: String = pallet.name.to_string();
				let event_type_id = event.ty.id;
				let event_variant = get_type_def_variant(event_type_id)?;
				for variant in &event_variant.variants {
					events.insert(
						(pallet.index, variant.index),
						EventMetadata {
							pallet: pallet_name.clone(),
							event: variant.name.to_owned(),
							fields: variant
								.fields
								.iter()
								.map(|f| {
									EventFieldMetadata::new(
										f.name.clone(),
										f.type_name.clone(),
										f.ty.id,
									)
								})
								.collect(),
							docs: variant.docs.to_vec(),
						},
					);
				}
			}
		}

		let mut errors = BTreeMap::<(u8, u8), ErrorMetadata>::new();
		for pallet in &metadata.pallets {
			if let Some(error) = &pallet.error {
				let pallet_name: String = pallet.name.to_string();
				let error_variant = get_type_def_variant(error.ty.id)?;
				for variant in &error_variant.variants {
					errors.insert(
						(pallet.index, variant.index),
						ErrorMetadata {
							pallet: pallet_name.clone(),
							error: variant.name.clone(),
							docs: variant.docs.to_vec(),
						},
					);
				}
			}
		}

		let dispatch_error_ty = metadata
			.types
			.types
			.iter()
			.find(|ty| ty.ty.path.segments == ["sp_runtime", "DispatchError"])
			.map(|ty| ty.id);

		Ok(Metadata { runtime_metadata: metadata, pallets, events, errors, dispatch_error_ty })
	}
}

/// Get the storage keys corresponding to a storage
///
/// This is **not** part of subxt.
impl Metadata {
	pub fn storage_value_key(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
	) -> Result<StorageKey, MetadataError> {
		Ok(self.pallet(pallet)?.storage(storage_item)?.get_value(pallet)?.key())
	}

	pub fn storage_map_key<K: Encode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		map_key: K,
	) -> Result<StorageKey, MetadataError> {
		Ok(self.pallet(pallet)?.storage(storage_item)?.get_map::<K>(pallet)?.key(map_key))
	}

	pub fn storage_map_key_prefix(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
	) -> Result<StorageKey, MetadataError> {
		self.pallet(pallet)?.storage(storage_item)?.get_map_prefix(pallet)
	}

	pub fn storage_double_map_key<K: Encode, Q: Encode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		first_double_map_key: K,
		second_double_map_key: Q,
	) -> Result<StorageKey, MetadataError> {
		Ok(self
			.pallet(pallet)?
			.storage(storage_item)?
			.get_double_map::<K, Q>(pallet)?
			.key(first_double_map_key, second_double_map_key))
	}
}
