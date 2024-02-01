// This file bases on subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG.

// Copyright 2019-2023 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! A representation of a block of events.

use crate::{
	error::{DispatchError, Error},
	events::{EventMetadataDetails, RootEvent},
	metadata::MetadataError,
	EventDetails, Metadata, Phase, StaticEvent,
};
use alloc::{
	string::{String, ToString},
	sync::Arc,
	vec::Vec,
};
use codec::{Decode, Encode};
use log::*;
use scale_decode::DecodeAsFields;
use scale_info::PortableRegistry;
use scale_value::{scale::TypeId, Composite};

/// Raw event details without the associated metadata
// Based on subxt EventDetails.
// https://github.com/paritytech/subxt/blob/8413c4d2dd625335b9200dc2289670accdf3391a/subxt/src/events/events_type.rs#L197-L216
#[derive(Debug, Clone, Encode, Decode)]
pub struct RawEventDetails<Hash: Decode + Encode> {
	phase: Phase,
	/// The index of the event in the list of events in a given block.
	index: u32,
	/// Raw event bytes, inlcuding
	/// - The phase.
	/// - Pallet and event index.
	/// - Event fields.
	/// - Event Topics.
	bytes: Vec<u8>,
	/// Raw event field bytes.
	field_bytes: Vec<u8>,
	/// Associated Pallet information.
	pallet_index: u8,
	pallet_name: String,
	pallet_bytes: Vec<u8>,
	/// Associated Variant information.
	variant_index: u8,
	variant_name: String,
	/// Associated Topcis.
	topics: Vec<Hash>,
}

impl<Hash: Encode + Decode> RawEventDetails<Hash> {
	/// When was the event produced?
	pub fn phase(&self) -> Phase {
		self.phase
	}

	/// What index is this event in the stored events for this block.
	pub fn index(&self) -> u32 {
		self.index
	}

	/// The index of the pallet that the event originated from.
	pub fn pallet_index(&self) -> u8 {
		// Note: never panics; we expect these bytes to exist
		// in order that the EventDetails could be created.
		self.pallet_index
	}

	/// The index of the event variant that the event originated from.
	pub fn variant_index(&self) -> u8 {
		// Note: never panics; we expect these bytes to exist
		// in order that the EventDetails could be created.
		self.variant_index
	}

	/// The name of the pallet from whence the Event originated.
	pub fn pallet_name(&self) -> &str {
		&self.pallet_name
	}

	/// The name of the event (ie the name of the variant that it corresponds to).
	pub fn variant_name(&self) -> &str {
		&self.variant_name
	}

	/// Fetch details from the metadata for this event.
	pub fn event_metadata<'a>(
		&'a self,
		metadata: &'a Metadata,
	) -> Result<EventMetadataDetails, Error> {
		let pallet = metadata
			.pallet_by_index(self.pallet_index())
			.ok_or(Error::Metadata(MetadataError::PalletIndexNotFound(self.pallet_index())))?;
		let variant = pallet
			.event_variant_by_index(self.variant_index())
			.ok_or(Error::Metadata(MetadataError::VariantIndexNotFound(self.variant_index())))?;
		let event_metadata = EventMetadataDetails { pallet, variant };
		self.metadata_sanity_check(&event_metadata)?;
		Ok(event_metadata)
	}

	/// Return _all_ of the bytes representing this event, which include, in order:
	/// - The phase.
	/// - Pallet and event index.
	/// - Event fields.
	/// - Event Topics.
	pub fn bytes(&self) -> &[u8] {
		&self.bytes
	}

	/// Return the bytes representing the fields stored in this event.
	pub fn field_bytes(&self) -> &[u8] {
		&self.field_bytes
	}

	/// Decode and provide the event fields back in the form of a [`scale_value::Composite`]
	/// type which represents the named or unnamed fields that were present in the event.
	pub fn field_values(&self, metadata: &Metadata) -> Result<Composite<TypeId>, Error> {
		let event_metadata = self.event_metadata(metadata)?;
		self.field_values_inner(&event_metadata, metadata.types())
	}

	/// Attempt to decode these [`EventDetails`] into a specific static event.
	/// This targets the fields within the event directly. You can also attempt to
	/// decode the entirety of the event type (including the pallet and event
	/// variants) using [`EventDetails::as_root_event()`].
	pub fn as_event<E: StaticEvent>(&self) -> Result<Option<E>, Error> {
		if self.pallet_name() == E::PALLET && self.variant_name() == E::EVENT {
			Ok(Some(E::decode(&mut self.field_bytes())?))
		} else {
			Ok(None)
		}
	}

	/// Attempt to decode these [`EventDetails`] into a root event type (which includes
	/// the pallet and event enum variants as well as the event fields). A compatible
	/// type for this is exposed via static codegen as a root level `Event` type.
	pub fn as_root_event<E: RootEvent>(&self, metadata: &Metadata) -> Result<E, Error> {
		let event_metadata = self.event_metadata(metadata)?;
		self.as_root_event_inner(metadata, &event_metadata)
	}

	/// Return the topics associated with this event.
	pub fn topics(&self) -> &[Hash] {
		&self.topics
	}

	pub fn pallet_bytes(&self) -> &[u8] {
		&self.pallet_bytes
	}
}

impl<Hash: Encode + Decode> RawEventDetails<Hash> {
	/// Checks if the extrinsic has failed.
	pub fn has_failed(&self) -> bool {
		self.pallet_name() == "System" && self.variant_name() == "ExtrinsicFailed"
	}

	/// Returns the dispatch error of the failed extrinsic, if it has failed.
	pub fn get_associated_dispatch_error(&self, metadata: &Metadata) -> Option<DispatchError> {
		match self.has_failed() {
			true => Some(
				DispatchError::decode_from(self.field_bytes(), metadata)
					.unwrap_or(DispatchError::CannotLookup),
			),
			false => None,
		}
	}

	/// Checks if the event represents a code update (runtime update).
	pub fn is_code_update(&self) -> bool {
		self.pallet_name() == "System" && self.variant_name() == "CodeUpdated"
	}
}

impl<Hash: Encode + Decode> From<EventDetails<Hash>> for RawEventDetails<Hash> {
	fn from(val: EventDetails<Hash>) -> Self {
		val.to_raw()
	}
}

// Private / Crate methods
impl<Hash: Encode + Decode> RawEventDetails<Hash> {
	// Attempt to dynamically decode a single event from our events input.
	pub(crate) fn decode_from(
		metadata: &Metadata,
		all_bytes: Arc<[u8]>,
		start_idx: usize,
		index: u32,
	) -> Result<Self, Error> {
		let input = &mut &all_bytes[start_idx..];

		let phase = Phase::decode(input)?;

		let event_start_idx = all_bytes.len() - input.len();

		let pallet_index = u8::decode(input)?;
		let variant_index = u8::decode(input)?;

		let event_fields_start_idx = all_bytes.len() - input.len();

		// Get metadata for the event:
		let event_pallet = metadata.pallet_by_index_err(pallet_index)?;
		let event_variant = event_pallet
			.event_variant_by_index(variant_index)
			.ok_or(MetadataError::VariantIndexNotFound(variant_index))?;
		let pallet_name = event_pallet.name().to_string();
		let variant_name = event_variant.name.to_string();
		debug!("Decoding Event '{}::{}'", &pallet_name, &variant_name);

		// Skip over the bytes belonging to this event.
		for field_metadata in &event_variant.fields {
			// Skip over the bytes for this field:
			scale_decode::visitor::decode_with_visitor(
				input,
				field_metadata.ty.id,
				metadata.types(),
				scale_decode::visitor::IgnoreVisitor,
			)
			.map_err(scale_decode::Error::from)?;
		}

		// the end of the field bytes.
		let event_fields_end_idx = all_bytes.len() - input.len();

		// topics come after the event data in EventRecord.
		let topics = Vec::<Hash>::decode(input)?;

		// what bytes did we skip over in total, including topics.
		let end_idx = all_bytes.len() - input.len();
		let pallet_bytes: Vec<u8> = all_bytes[event_start_idx + 1..event_fields_end_idx].into();
		let pallet_index = all_bytes[event_fields_start_idx - 2];
		let variant_index = all_bytes[event_fields_start_idx - 1];
		let bytes: Vec<u8> = all_bytes[start_idx..end_idx].into();
		let field_bytes: Vec<u8> = all_bytes[event_fields_start_idx..event_fields_end_idx].into();

		Ok(RawEventDetails {
			phase,
			index,
			bytes,
			field_bytes,
			pallet_index,
			pallet_name,
			pallet_bytes,
			variant_index,
			variant_name,
			topics,
		})
	}

	/// Fetch details from the metadata for this event.
	pub(crate) fn event_metadata_unchecked<'a>(
		&'a self,
		metadata: &'a Metadata,
	) -> EventMetadataDetails {
		let pallet = metadata
			.pallet_by_index(self.pallet_index())
			.expect("event pallet to be found; we did this already during decoding");
		let variant = pallet
			.event_variant_by_index(self.variant_index())
			.expect("event variant to be found; we did this already during decoding");

		EventMetadataDetails { pallet, variant }
	}

	/// Decode and provide the event fields back in the form of a [`scale_value::Composite`]
	/// type which represents the named or unnamed fields that were present in the event.
	pub(crate) fn field_values_unchecked(
		&self,
		metadata: &Metadata,
	) -> Result<Composite<TypeId>, Error> {
		let event_metadata = self.event_metadata_unchecked(metadata);
		self.field_values_inner(&event_metadata, metadata.types())
	}

	/// Attempt to decode these [`EventDetails`] into a root event type (which includes
	/// the pallet and event enum variants as well as the event fields). A compatible
	/// type for this is exposed via static codegen as a root level `Event` type.
	pub(crate) fn as_root_event_unchecked<E: RootEvent>(
		&self,
		metadata: &Metadata,
	) -> Result<E, Error> {
		let event_metadata = self.event_metadata_unchecked(metadata);
		self.as_root_event_inner(metadata, &event_metadata)
	}

	pub(crate) fn as_root_event_inner<E: RootEvent>(
		&self,
		metadata: &Metadata,
		event_metadata: &EventMetadataDetails,
	) -> Result<E, Error> {
		let pallet_bytes = self.pallet_bytes();
		let pallet_event_ty = event_metadata.pallet.event_ty_id().ok_or_else(|| {
			MetadataError::EventTypeNotFoundInPallet(event_metadata.pallet.index())
		})?;

		E::root_event(pallet_bytes, self.pallet_name(), pallet_event_ty, metadata)
	}

	fn metadata_sanity_check(&self, event_metadata: &EventMetadataDetails) -> Result<(), Error> {
		if event_metadata.pallet.name() != self.pallet_name()
			|| event_metadata.variant.name != self.variant_name()
		{
			return Err(Error::Metadata(MetadataError::MetadataMismatch))
		}
		Ok(())
	}

	fn field_values_inner(
		&self,
		event_metadata: &EventMetadataDetails,
		types: &PortableRegistry,
	) -> Result<Composite<TypeId>, Error> {
		let bytes = &mut self.field_bytes();
		let mut fields = event_metadata
			.variant
			.fields
			.iter()
			.map(|f| scale_decode::Field::new(f.ty.id, f.name.as_deref()));
		let decoded = <Composite<TypeId>>::decode_as_fields(bytes, &mut fields, types)?;

		Ok(decoded)
	}
}
