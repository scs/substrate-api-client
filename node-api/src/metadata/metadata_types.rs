// This file bases on subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG.
//
// Copyright 2019-2023 Parity Technologies (UK) Ltd and Supercomputing Systems AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! Handle substrate chain metadata.

use crate::{
	metadata::{v14_to_v15, variant_index::VariantIndex, MetadataConversionError, MetadataError},
	storage::GetStorageTypes,
};
use alloc::{
	collections::btree_map::BTreeMap,
	string::{String, ToString},
	vec::Vec,
};
use codec::{Decode, Encode};
use frame_metadata::{
	v15::{
		CustomMetadata, ExtrinsicMetadata, OuterEnums, PalletConstantMetadata,
		RuntimeApiMethodMetadata, RuntimeMetadataLastVersion, StorageEntryMetadata,
	},
	RuntimeMetadata, RuntimeMetadataPrefixed, META_RESERVED,
};
use scale_info::{
	form::{Form, PortableForm},
	PortableRegistry, Type, Variant,
};
use sp_storage::StorageKey;

#[cfg(feature = "std")]
use serde::Serialize;

/// Metadata wrapper around the runtime metadata. Offers some extra features,
/// such as direct pallets, events and error access.
#[derive(Clone, Debug)]
pub struct Metadata {
	runtime_metadata: RuntimeMetadataLastVersion,
	pallets: BTreeMap<String, PalletMetadataInner>,
	/// Find the location in the pallet Vec by pallet index.
	pallets_by_index: BTreeMap<u8, String>,
	// Type of the DispatchError type, which is what comes back if
	// an extrinsic fails.
	dispatch_error_ty: Option<u32>,
	/// Details about each of the runtime API traits.
	apis: BTreeMap<String, RuntimeApiMetadataInner>,
}

impl Metadata {
	/// An iterator over all of the available pallets.
	pub fn pallets(&self) -> impl Iterator<Item = PalletMetadata<'_>> {
		self.pallets.values().map(|inner| PalletMetadata { inner, types: self.types() })
	}

	/// Access a pallet given its encoded variant index.
	pub fn pallet_by_index(&self, variant_index: u8) -> Option<PalletMetadata<'_>> {
		let name = self.pallets_by_index.get(&variant_index)?;
		self.pallet_by_name(name)
	}

	/// Access a pallet given its name.
	pub fn pallet_by_name(&self, pallet_name: &str) -> Option<PalletMetadata<'_>> {
		let inner = self.pallets.get(pallet_name)?;

		Some(PalletMetadata { inner, types: self.types() })
	}

	/// Return the type of the `Runtime`.
	pub fn ty(&self) -> &<PortableForm as Form>::Type {
		&self.runtime_metadata.ty
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

	/// Exposes the runtime metadata.
	pub fn runtime_metadata(&self) -> &RuntimeMetadataLastVersion {
		&self.runtime_metadata
	}

	/// Return details about the extrinsic format.
	pub fn extrinsic(&self) -> &ExtrinsicMetadata<PortableForm> {
		&self.runtime_metadata.extrinsic
	}

	/// An iterator over all of the runtime APIs.
	pub fn runtime_api_traits(&self) -> impl ExactSizeIterator<Item = RuntimeApiMetadata<'_>> {
		self.apis
			.values()
			.map(|inner| RuntimeApiMetadata { inner, types: self.types() })
	}

	/// Access a runtime API trait given its name.
	pub fn runtime_api_trait_by_name(&'_ self, name: &str) -> Option<RuntimeApiMetadata<'_>> {
		let inner = self.apis.get(name)?;
		Some(RuntimeApiMetadata { inner, types: self.types() })
	}

	/// Return the outer enums types as found in the runtime.
	pub fn outer_enums(&self) -> &OuterEnums<PortableForm> {
		&self.runtime_metadata.outer_enums
	}

	/// Returns the custom types of the metadata.
	pub fn custom(&self) -> &CustomMetadata<PortableForm> {
		&self.runtime_metadata.custom
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

/// Err wrappers around option.
impl Metadata {
	/// Identical to `metadata.pallet_by_name()`, but returns an error if the pallet is not found.
	pub fn pallet_by_name_err(&self, name: &str) -> Result<PalletMetadata<'_>, MetadataError> {
		self.pallet_by_name(name)
			.ok_or_else(|| MetadataError::PalletNameNotFound(name.to_string()))
	}

	/// Identical to `metadata.pallet_by_index()`, but returns an error if the pallet is not found.
	pub fn pallet_by_index_err(&self, index: u8) -> Result<PalletMetadata<'_>, MetadataError> {
		self.pallet_by_index(index).ok_or(MetadataError::PalletIndexNotFound(index))
	}

	/// Identical to `metadata.runtime_api_trait_by_name()`, but returns an error if the trait is not found.
	pub fn runtime_api_trait_by_name_err(
		&self,
		name: &str,
	) -> Result<RuntimeApiMetadata<'_>, MetadataError> {
		self.runtime_api_trait_by_name(name)
			.ok_or_else(|| MetadataError::RuntimeApiNotFound(name.to_string()))
	}
}

/// Metadata for a specific pallet.
// Based on https://github.com/paritytech/subxt/blob/8413c4d2dd625335b9200dc2289670accdf3391a/metadata/src/lib.rs#L153-L251
#[derive(Debug, Clone, Copy)]
pub struct PalletMetadata<'a> {
	inner: &'a PalletMetadataInner,
	types: &'a PortableRegistry,
}

impl<'a> PalletMetadata<'a> {
	/// The pallet name.
	pub fn name(&self) -> &'a str {
		&self.inner.name
	}

	/// The pallet index.
	pub fn index(&self) -> u8 {
		self.inner.index
	}

	/// The pallet docs.
	pub fn docs(&self) -> &'a [String] {
		&self.inner.docs
	}

	/// Type ID for the pallet's Call type, if it exists.
	pub fn call_ty_id(&self) -> Option<u32> {
		self.inner.call_ty
	}

	/// Type ID for the pallet's Event type, if it exists.
	pub fn event_ty_id(&self) -> Option<u32> {
		self.inner.event_ty
	}

	/// Type ID for the pallet's Error type, if it exists.
	pub fn error_ty_id(&self) -> Option<u32> {
		self.inner.error_ty
	}

	/// An iterator over the constants in this pallet.
	pub fn storage(&self) -> impl ExactSizeIterator<Item = &'a StorageEntryMetadata<PortableForm>> {
		self.inner.storage.values()
	}
	/// Return metadata storage entry data for given key.
	pub fn storage_entry(
		&self,
		key: &'static str,
	) -> Result<&StorageEntryMetadata<PortableForm>, MetadataError> {
		self.inner.storage.get(key).ok_or(MetadataError::StorageNotFound(key))
	}

	/// Return all of the event variants, if an event type exists.
	pub fn event_variants(&self) -> Option<&'a [Variant<PortableForm>]> {
		VariantIndex::get(self.inner.event_ty, self.types)
	}

	/// Return an event variant given it's encoded variant index.
	pub fn event_variant_by_index(&self, variant_index: u8) -> Option<&'a Variant<PortableForm>> {
		self.inner.event_variant_index.lookup_by_index(
			variant_index,
			self.inner.event_ty,
			self.types,
		)
	}

	/// Return all of the call variants, if a call type exists.
	pub fn call_variants(&self) -> Option<&'a [Variant<PortableForm>]> {
		VariantIndex::get(self.inner.call_ty, self.types)
	}

	/// Return a call variant given it's encoded variant index.
	pub fn call_variant_by_index(&self, variant_index: u8) -> Option<&'a Variant<PortableForm>> {
		self.inner
			.call_variant_index
			.lookup_by_index(variant_index, self.inner.call_ty, self.types)
	}

	/// Return a call variant given it's name.
	pub fn call_variant_by_name(&self, call_name: &str) -> Option<&'a Variant<PortableForm>> {
		self.inner
			.call_variant_index
			.lookup_by_name(call_name, self.inner.call_ty, self.types)
	}

	/// Return all of the error variants, if an error type exists.
	pub fn error_variants(&self) -> Option<&'a [Variant<PortableForm>]> {
		VariantIndex::get(self.inner.error_ty, self.types)
	}

	/// Return an error variant given it's encoded variant index.
	pub fn error_variant_by_index(&self, variant_index: u8) -> Option<&'a Variant<PortableForm>> {
		self.inner.error_variant_index.lookup_by_index(
			variant_index,
			self.inner.error_ty,
			self.types,
		)
	}

	/// Return constant details given the constant name.
	pub fn constant_by_name(&self, name: &str) -> Option<&'a PalletConstantMetadata<PortableForm>> {
		self.inner.constants.get(name)
	}

	/// An iterator over the constants in this pallet.
	pub fn constants(
		&self,
	) -> impl ExactSizeIterator<Item = &'a PalletConstantMetadata<PortableForm>> {
		self.inner.constants.values()
	}
}

// Based on https://github.com/paritytech/frame-metadata/blob/94e7743fa454963609763cf9cccbb7f85bc96d2f/frame-metadata/src/v15.rs#L249-L276
#[derive(Debug, Clone)]
struct PalletMetadataInner {
	/// Pallet name.
	name: String,
	/// Pallet storage metadata.
	storage: BTreeMap<String, StorageEntryMetadata<PortableForm>>,
	/// Type ID for the pallet Call enum.
	call_ty: Option<u32>,
	/// Call variants by name/u8.
	call_variant_index: VariantIndex,
	/// Type ID for the pallet Event enum.
	event_ty: Option<u32>,
	/// Event variants by name/u8.
	event_variant_index: VariantIndex,
	/// Map from constant name to constant details.
	constants: BTreeMap<String, PalletConstantMetadata<PortableForm>>,
	/// Type ID for the pallet Error enum.
	error_ty: Option<u32>,
	/// Error variants by name/u8.
	error_variant_index: VariantIndex,
	/// Define the index of the pallet, this index will be used for the encoding of pallet event,
	/// call and origin variants.
	index: u8,
	/// Pallet documentation.
	docs: Vec<String>,
}

/// Metadata for the available runtime APIs.
// Based on https://github.com/paritytech/subxt/blob/8413c4d2dd625335b9200dc2289670accdf3391a/metadata/src/lib.rs#L494-L527
#[derive(Debug, Clone, Copy)]
pub struct RuntimeApiMetadata<'a> {
	inner: &'a RuntimeApiMetadataInner,
	types: &'a PortableRegistry,
}

impl<'a> RuntimeApiMetadata<'a> {
	/// Trait name.
	pub fn name(&self) -> &'a str {
		&self.inner.name
	}
	/// Trait documentation.
	pub fn docs(&self) -> &[String] {
		&self.inner.docs
	}
	/// Return the type registry embedded within the metadata.
	pub fn types(&self) -> &'a PortableRegistry {
		self.types
	}
	/// An iterator over the trait methods.
	pub fn methods(
		&self,
	) -> impl ExactSizeIterator<Item = &'a RuntimeApiMethodMetadata<PortableForm>> {
		self.inner.methods.values()
	}
	/// Get a specific trait method given its name.
	pub fn method_by_name(&self, name: &str) -> Option<&'a RuntimeApiMethodMetadata<PortableForm>> {
		self.inner.methods.get(name)
	}
}

// Based on https://github.com/paritytech/subxt/blob/8413c4d2dd625335b9200dc2289670accdf3391a/metadata/src/lib.rs#L529-L537
#[derive(Debug, Clone)]
struct RuntimeApiMetadataInner {
	/// Trait name.
	name: String,
	/// Trait methods.
	methods: BTreeMap<String, RuntimeApiMethodMetadata<PortableForm>>,
	/// Trait documentation.
	docs: Vec<String>,
}

// Based on https://github.com/paritytech/subxt/blob/8413c4d2dd625335b9200dc2289670accdf3391a/metadata/src/from_into/v15.rs
impl TryFrom<RuntimeMetadataPrefixed> for Metadata {
	type Error = MetadataConversionError;

	fn try_from(m: RuntimeMetadataPrefixed) -> Result<Self, Self::Error> {
		if m.0 != META_RESERVED {
			return Err(MetadataConversionError::InvalidPrefix)
		}

		let m = match m.1 {
			RuntimeMetadata::V14(meta) => v14_to_v15(meta)?,
			RuntimeMetadata::V15(meta) => meta,
			_ => return Err(MetadataConversionError::InvalidVersion),
		};

		let mut pallets = BTreeMap::new();
		let mut pallets_by_index = BTreeMap::new();
		for p in m.pallets.clone().into_iter() {
			let name = p.name;

			let storage = p.storage.as_ref().map_or(BTreeMap::new(), |storage| {
				storage
					.entries
					.iter()
					.map(|entry| (entry.name.clone(), entry.clone()))
					.collect()
			});

			let constants = p
				.constants
				.iter()
				.map(|constant| (constant.name.clone(), constant.clone()))
				.collect();

			let call_variant_index =
				VariantIndex::build(p.calls.as_ref().map(|c| c.ty.id), &m.types);
			let error_variant_index =
				VariantIndex::build(p.error.as_ref().map(|e| e.ty.id), &m.types);
			let event_variant_index =
				VariantIndex::build(p.event.as_ref().map(|e| e.ty.id), &m.types);

			pallets_by_index.insert(p.index, name.clone());
			pallets.insert(
				name.clone(),
				PalletMetadataInner {
					name,
					index: p.index,
					storage,
					call_ty: p.calls.map(|c| c.ty.id),
					call_variant_index,
					event_ty: p.event.map(|e| e.ty.id),
					event_variant_index,
					error_ty: p.error.map(|e| e.ty.id),
					error_variant_index,
					constants,
					docs: p.docs,
				},
			);
		}

		let apis = m
			.apis
			.iter()
			.map(|api| {
				(api.name.clone(), {
					let name = api.name.clone();
					let docs = api.docs.clone();
					let methods = api
						.methods
						.iter()
						.map(|method| (method.name.clone(), method.clone()))
						.collect();
					RuntimeApiMetadataInner { name, docs, methods }
				})
			})
			.collect();

		let dispatch_error_ty = m
			.types
			.types
			.iter()
			.find(|ty| ty.ty.path.segments == ["sp_runtime", "DispatchError"])
			.map(|ty| ty.id);

		Ok(Metadata {
			runtime_metadata: m.clone(),
			pallets,
			pallets_by_index,
			dispatch_error_ty,
			apis,
		})
	}
}

// Support decoding metadata from the "wire" format directly into this.
// Errors may be lost in the case that the metadata content is somehow invalid.
impl Decode for Metadata {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let metadata = frame_metadata::RuntimeMetadataPrefixed::decode(input)?;
		metadata.try_into().map_err(|_e| "Cannot try_into() to Metadata.".into())
	}
}

// Metadata can be encoded, too. It will encode into a format that's compatible with what
// Subxt requires, and that it can be decoded back from. The actual specifics of the format
// can change over time.
impl Encode for Metadata {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		let m: frame_metadata::v15::RuntimeMetadataV15 = self.runtime_metadata().clone();
		let m: frame_metadata::RuntimeMetadataPrefixed = m.into();
		m.encode_to(dest)
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
		Ok(self
			.pallet_by_name_err(pallet)?
			.storage_entry(storage_item)?
			.get_value(pallet)?
			.key())
	}

	pub fn storage_map_key<K: Encode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		map_key: K,
	) -> Result<StorageKey, MetadataError> {
		Ok(self
			.pallet_by_name_err(pallet)?
			.storage_entry(storage_item)?
			.get_map::<K>(pallet)?
			.key(map_key))
	}

	pub fn storage_map_key_prefix(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
	) -> Result<StorageKey, MetadataError> {
		self.pallet_by_name_err(pallet)?
			.storage_entry(storage_item)?
			.get_map_prefix(pallet)
	}

	pub fn storage_double_map_key_prefix<K: Encode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
	) -> Result<StorageKey, MetadataError> {
		self.pallet_by_name_err(storage_prefix)?
			.storage_entry(storage_key_name)?
			.get_double_map_prefix::<K>(storage_prefix, first)
	}

	pub fn storage_double_map_key<K: Encode, Q: Encode>(
		&self,
		pallet: &'static str,
		storage_item: &'static str,
		first_double_map_key: K,
		second_double_map_key: Q,
	) -> Result<StorageKey, MetadataError> {
		Ok(self
			.pallet_by_name_err(pallet)?
			.storage_entry(storage_item)?
			.get_double_map::<K, Q>(pallet)?
			.key(first_double_map_key, second_double_map_key))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use codec::Decode;
	use scale_info::TypeDef;
	use sp_core::Bytes;
	use std::fs;

	fn metadata() -> Metadata {
		let encoded_metadata: Bytes = fs::read("./../ksm_metadata_v14.bin").unwrap().into();
		Decode::decode(&mut encoded_metadata.0.as_slice()).unwrap()
	}

	#[test]
	fn outer_enum_access() {
		let metadata = metadata();

		let call_enum_ty = metadata.outer_enums().call_enum_ty;
		let ty = metadata.types().types.get(call_enum_ty.id as usize).unwrap();
		if let TypeDef::Variant(variant) = &ty.ty.type_def {
			// The first pallet call is from System pallet.
			assert_eq!(variant.variants[0].name, "System");
		} else {
			panic!("Expected Variant outer enum call type.");
		}
	}

	#[test]
	fn custom_ksm_metadata_v14_is_empty() {
		let metadata = metadata();
		let custom_metadata = metadata.custom();

		assert!(custom_metadata.map.is_empty());
	}
}
