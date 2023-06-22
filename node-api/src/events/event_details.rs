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

use crate::{
	error::{DispatchError, Error},
	metadata::EventMetadata,
	scale_value::{decode_as_type, Composite, TypeId},
	Metadata, Phase, StaticEvent,
};
use alloc::{string::ToString, sync::Arc, vec, vec::Vec};
use codec::{Decode, Error as CodecError};
use log::*;
use sp_core::H256;

/// The event details.
#[derive(Debug, Clone)]
pub struct EventDetails {
	phase: Phase,
	index: u32,
	all_bytes: Arc<[u8]>,
	// start of the bytes (phase, pallet/variant index and then fields and then topic to follow).
	start_idx: usize,
	// start of the fields (ie after phase nad pallet/variant index).
	fields_start_idx: usize,
	// end of the fields.
	fields_end_idx: usize,
	// end of everything (fields + topics)
	end_idx: usize,
	metadata: Metadata,
}

impl EventDetails {
	// Attempt to dynamically decode a single event from our events input.
	pub(crate) fn decode_from(
		metadata: Metadata,
		all_bytes: Arc<[u8]>,
		start_idx: usize,
		index: u32,
	) -> Result<EventDetails, Error> {
		let input = &mut &all_bytes[start_idx..];

		let phase = Phase::decode(input)?;
		let pallet_index = u8::decode(input)?;
		let variant_index = u8::decode(input)?;

		let fields_start_idx = all_bytes.len() - input.len();

		// Get metadata for the event:
		let event_metadata = metadata.event(pallet_index, variant_index)?;
		debug!("Decoding Event '{}::{}'", event_metadata.pallet(), event_metadata.event());

		// Skip over the bytes belonging to this event.
		for field_metadata in event_metadata.fields() {
			// Skip over the bytes for this field:
			decode_as_type(input, field_metadata.type_id(), &metadata.runtime_metadata().types)?;
		}

		// the end of the field bytes.
		let fields_end_idx = all_bytes.len() - input.len();

		// topics come after the event data in EventRecord. They aren't used for
		// anything at the moment, so just decode and throw them away.
		let _topics = Vec::<H256>::decode(input)?;

		// what bytes did we skip over in total, including topics.
		let end_idx = all_bytes.len() - input.len();

		Ok(EventDetails {
			phase,
			index,
			start_idx,
			fields_start_idx,
			fields_end_idx,
			end_idx,
			all_bytes,
			metadata,
		})
	}

	/// When was the event produced?
	pub fn phase(&self) -> Phase {
		self.phase.clone()
	}

	/// What index is this event in the stored events for this block.
	pub fn index(&self) -> u32 {
		self.index
	}

	/// The index of the pallet that the event originated from.
	pub fn pallet_index(&self) -> u8 {
		// Note: never panics; we expect these bytes to exist
		// in order that the EventDetails could be created.
		self.all_bytes[self.fields_start_idx - 2]
	}

	/// The index of the event variant that the event originated from.
	pub fn variant_index(&self) -> u8 {
		// Note: never panics; we expect these bytes to exist
		// in order that the EventDetails could be created.
		self.all_bytes[self.fields_start_idx - 1]
	}

	/// The name of the pallet from whence the Event originated.
	pub fn pallet_name(&self) -> &str {
		self.event_metadata().pallet()
	}

	/// The name of the event (ie the name of the variant that it corresponds to).
	pub fn variant_name(&self) -> &str {
		self.event_metadata().event()
	}

	/// Fetch the metadata for this event.
	pub fn event_metadata(&self) -> &EventMetadata {
		self.metadata
			.event(self.pallet_index(), self.variant_index())
			.expect("this must exist in order to have produced the EventDetails")
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
		&self.all_bytes[self.fields_start_idx..self.fields_end_idx]
	}

	/// Decode and provide the event fields back in the form of a [`scale_value::Composite`]
	/// type which represents the named or unnamed fields that were
	/// present in the event.
	pub fn field_values(&self) -> Result<Composite<TypeId>, Error> {
		let bytes = &mut self.field_bytes();
		let event_metadata = self.event_metadata();

		// If the first field has a name, we assume that the rest do too (it'll either
		// be a named struct or a tuple type). If no fields, assume unnamed.
		let is_named =
			event_metadata.fields().get(0).map(|fm| fm.name().is_some()).unwrap_or(false);

		if !is_named {
			let mut event_values = vec![];
			for field_metadata in event_metadata.fields() {
				let value = decode_as_type(
					bytes,
					field_metadata.type_id(),
					&self.metadata.runtime_metadata().types,
				)?;
				event_values.push(value);
			}

			Ok(Composite::Unnamed(event_values))
		} else {
			let mut event_values = vec![];
			for field_metadata in event_metadata.fields() {
				let value = decode_as_type(
					bytes,
					field_metadata.type_id(),
					&self.metadata.runtime_metadata().types,
				)?;
				event_values.push((field_metadata.name().unwrap_or_default().to_string(), value));
			}

			Ok(Composite::Named(event_values))
		}
	}

	/// Attempt to decode these [`EventDetails`] into a specific static event.
	/// This targets the fields within the event directly. You can also attempt to
	/// decode the entirety of the event type (including the pallet and event
	/// variants) using [`EventDetails::as_root_event()`].
	pub fn as_event<E: StaticEvent>(&self) -> Result<Option<E>, CodecError> {
		let ev_metadata = self.event_metadata();
		if ev_metadata.pallet() == E::PALLET && ev_metadata.event() == E::EVENT {
			Ok(Some(E::decode(&mut self.field_bytes())?))
		} else {
			Ok(None)
		}
	}

	/// Attempt to decode these [`EventDetails`] into a root event type (which includes
	/// the pallet and event enum variants as well as the event fields). A compatible
	/// type for this is exposed via static codegen as a root level `Event` type.
	pub fn as_root_event<E: Decode>(&self) -> Result<E, CodecError> {
		E::decode(&mut self.bytes())
	}
}

impl EventDetails {
	/// Checks if the extrinsic has failed. If so, the corresponding DispatchError is returned.
	pub fn check_if_failed(&self) -> Result<(), DispatchError> {
		if self.pallet_name() == "System" && self.variant_name() == "ExtrinsicFailed" {
			let dispatch_error = DispatchError::decode_from(self.field_bytes(), &self.metadata);
			return Err(dispatch_error)
		}
		Ok(())
	}
}
