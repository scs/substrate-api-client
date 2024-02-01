// This file bases on subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG.

// Copyright 2019-2023 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! A representation of a block of events.

use crate::{
	error::{DispatchError, Error},
	metadata::MetadataError,
	Metadata, Phase, StaticEvent,
};
use alloc::{
	string::{String, ToString},
	sync::Arc,
	vec::Vec,
};
use codec::{Decode, Encode};
use log::*;

/// Raw event details without the associated metadata
// Based on subxt EventDetails.
// https://github.com/paritytech/subxt/blob/8413c4d2dd625335b9200dc2289670accdf3391a/subxt/src/events/events_type.rs#L197-L216
#[derive(Debug, Clone, Encode, Decode)]
pub struct RawEventDetails<Hash: Decode + Encode> {
	phase: Phase,
	/// The index of the event in the list of events in a given block.
	index: u32,
	//all_bytes: Arc<[u8]>,

	// // start of the bytes (phase, pallet/variant index and then fields and then topic to follow).
	// start_idx: u8,
	// // start of the event (ie pallet/variant index and then the fields and topic after).
	// event_start_idx: u8,
	// // start of the fields (ie after phase and pallet/variant index).
	// event_fields_start_idx: u8,
	// // end of the fields.
	// event_fields_end_idx: u8,
	// // end of everything (fields + topics)
	// end_idx: u8,
	bytes: Vec<u8>,
	field_bytes: Vec<u8>,
	pallet_index: u8,
	pallet_name: String,
	variant_index: u8,
	variant_name: String,
	pallet_bytes: Vec<u8>,
	topics: Vec<Hash>,
}

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
			//all_bytes,
			// start_idx: start_idx.into(),
			// event_start_idx: event_start_idx.into(),
			// event_fields_start_idx: event_fields_start_idx.into(),
			// event_fields_end_idx: event_fields_end_idx.into(),
			// end_idx: end_idx.into(),
			bytes,
			field_bytes,
			pallet_index,
			pallet_name,
			variant_index,
			variant_name,
			pallet_bytes,
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
	pub fn check_if_failed(&self) -> bool {
		self.pallet_name() == "System" && self.variant_name() == "ExtrinsicFailed"
	}

	/// Returns the dispatch error of the failed extrinsic, if it has failed.
	pub fn associated_dispatch_error(&self, metadata: &Metadata) -> Option<DispatchError> {
		match self.check_if_failed() {
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
