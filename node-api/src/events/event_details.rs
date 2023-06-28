// This file bases on subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG.

// Copyright 2019-2023 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! A representation of a block of events.

use crate::{
	error::{DispatchError, Error},
	metadata::{MetadataError, PalletMetadata},
	scale_value::{Composite, TypeId},
	Metadata, Phase, StaticEvent,
};
use alloc::{sync::Arc, vec::Vec};
use codec::Decode;
use log::*;
use scale_decode::DecodeAsFields;

/// The event details.
/// Based on subxt EventDetails.
/// https://github.com/paritytech/subxt/blob/8413c4d2dd625335b9200dc2289670accdf3391a/subxt/src/events/events_type.rs#L197-L216
#[derive(Debug, Clone)]
pub struct EventDetails<Hash: Decode> {
	phase: Phase,
	/// The index of the event in the list of events in a given block.
	index: u32,
	all_bytes: Arc<[u8]>,
	// start of the bytes (phase, pallet/variant index and then fields and then topic to follow).
	start_idx: usize,
	// start of the event (ie pallet/variant index and then the fields and topic after).
	event_start_idx: usize,
	// start of the fields (ie after phase and pallet/variant index).
	event_fields_start_idx: usize,
	// end of the fields.
	event_fields_end_idx: usize,
	// end of everything (fields + topics)
	end_idx: usize,
	metadata: Metadata,
	topics: Vec<Hash>,
}

// Based on subxt:
// https://github.com/paritytech/subxt/blob/8413c4d2dd625335b9200dc2289670accdf3391a/subxt/src/events/events_type.rs#L218-L409
impl<Hash: Decode> EventDetails<Hash> {
	// Attempt to dynamically decode a single event from our events input.
	pub(crate) fn decode_from(
		metadata: Metadata,
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
		debug!("Decoding Event '{}::{}'", event_pallet.name(), &event_variant.name);

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

		Ok(EventDetails {
			phase,
			index,
			start_idx,
			event_start_idx,
			event_fields_start_idx,
			event_fields_end_idx,
			end_idx,
			all_bytes,
			metadata,
			topics,
		})
	}

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
		self.all_bytes[self.event_fields_start_idx - 2]
	}

	/// The index of the event variant that the event originated from.
	pub fn variant_index(&self) -> u8 {
		// Note: never panics; we expect these bytes to exist
		// in order that the EventDetails could be created.
		self.all_bytes[self.event_fields_start_idx - 1]
	}

	/// The name of the pallet from whence the Event originated.
	pub fn pallet_name(&self) -> &str {
		self.event_metadata().pallet.name()
	}

	/// The name of the event (ie the name of the variant that it corresponds to).
	pub fn variant_name(&self) -> &str {
		&self.event_metadata().variant.name
	}

	/// Fetch details from the metadata for this event.
	pub fn event_metadata(&self) -> EventMetadataDetails {
		let pallet = self
			.metadata
			.pallet_by_index(self.pallet_index())
			.expect("event pallet to be found; we did this already during decoding");
		let variant = pallet
			.event_variant_by_index(self.variant_index())
			.expect("event variant to be found; we did this already during decoding");

		EventMetadataDetails { pallet, variant }
	}

	/// Return _all_ of the bytes representing this event, which include, in order:
	/// - The phase.
	/// - Pallet and event index.
	/// - Event fields.
	/// - Event Topics.
	pub fn bytes(&self) -> &[u8] {
		&self.all_bytes[self.start_idx..self.end_idx]
	}

	/// Return the bytes representing the fields stored in this event.
	pub fn field_bytes(&self) -> &[u8] {
		&self.all_bytes[self.event_fields_start_idx..self.event_fields_end_idx]
	}

	/// Decode and provide the event fields back in the form of a [`scale_value::Composite`]
	/// type which represents the named or unnamed fields that were present in the event.
	pub fn field_values(&self) -> Result<Composite<TypeId>, Error> {
		let bytes = &mut self.field_bytes();
		let event_metadata = self.event_metadata();

		let mut fields = event_metadata
			.variant
			.fields
			.iter()
			.map(|f| scale_decode::Field::new(f.ty.id, f.name.as_deref()));

		let decoded =
			<Composite<TypeId>>::decode_as_fields(bytes, &mut fields, self.metadata.types())?;

		Ok(decoded)
	}

	/// Attempt to decode these [`EventDetails`] into a specific static event.
	/// This targets the fields within the event directly. You can also attempt to
	/// decode the entirety of the event type (including the pallet and event
	/// variants) using [`EventDetails::as_root_event()`].
	pub fn as_event<E: StaticEvent>(&self) -> Result<Option<E>, Error> {
		let ev_metadata = self.event_metadata();
		if ev_metadata.pallet.name() == E::PALLET && ev_metadata.variant.name == E::EVENT {
			Ok(Some(E::decode(&mut self.field_bytes())?))
		} else {
			Ok(None)
		}
	}

	/// Attempt to decode these [`EventDetails`] into a root event type (which includes
	/// the pallet and event enum variants as well as the event fields). A compatible
	/// type for this is exposed via static codegen as a root level `Event` type.
	pub fn as_root_event<E: RootEvent>(&self) -> Result<E, Error> {
		let ev_metadata = self.event_metadata();
		let pallet_bytes = &self.all_bytes[self.event_start_idx + 1..self.event_fields_end_idx];
		let pallet_event_ty = ev_metadata
			.pallet
			.event_ty_id()
			.ok_or_else(|| MetadataError::EventTypeNotFoundInPallet(ev_metadata.pallet.index()))?;

		E::root_event(pallet_bytes, self.pallet_name(), pallet_event_ty, &self.metadata)
	}

	/// Return the topics associated with this event.
	pub fn topics(&self) -> &[Hash] {
		&self.topics
	}
}

impl<Hash: Decode> EventDetails<Hash> {
	/// Checks if the extrinsic has failed. If so, the corresponding DispatchError is returned.
	pub fn check_if_failed(&self) -> Result<(), DispatchError> {
		if self.pallet_name() == "System" && self.variant_name() == "ExtrinsicFailed" {
			let dispatch_error =
				DispatchError::decode_from(self.field_bytes(), self.metadata.clone())
					.map_err(|_| DispatchError::CannotLookup)?;
			return Err(dispatch_error)
		}
		Ok(())
	}
}

/// Details for the given event plucked from the metadata.
// Based on https://github.com/paritytech/subxt/blob/8413c4d2dd625335b9200dc2289670accdf3391a/subxt/src/events/events_type.rs#L411-L415
pub struct EventMetadataDetails<'a> {
	pub pallet: PalletMetadata<'a>,
	pub variant: &'a scale_info::Variant<scale_info::form::PortableForm>,
}

/// This trait is implemented on the statically generated root event type, so that we're able
/// to decode it properly via a pallet event that impls `DecodeAsMetadata`. This is necessary
/// becasue the "root event" type is generated using pallet info but doesn't actually exist in the
/// metadata types, so we have no easy way to decode things into it via type information and need a
/// little help via codegen.
// Based on https://github.com/paritytech/subxt/blob/8413c4d2dd625335b9200dc2289670accdf3391a/subxt/src/events/events_type.rs#L417-L432
#[doc(hidden)]
pub trait RootEvent: Sized {
	/// Given details of the pallet event we want to decode, and the name of the pallet, try to hand
	/// back a "root event".
	fn root_event(
		pallet_bytes: &[u8],
		pallet_name: &str,
		pallet_event_ty: u32,
		metadata: &Metadata,
	) -> Result<Self, Error>;
}
