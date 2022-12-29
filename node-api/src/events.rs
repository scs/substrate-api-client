// This file was taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! A representation of a block of events.
//!
//! This file is very similar to subxt, except where noted.
//! Based on https://github.com/paritytech/subxt/commit/1e8d0956cc6aeb882637bde1d09ac44186181781#

use crate::{
	alloc::{string::ToString, sync::Arc, vec, vec::Vec},
	decoder::{decode_as_type, Composite, TypeId},
	error::{DispatchError, Error},
	metadata::EventMetadata,
	Metadata, Phase, StaticEvent,
};
use codec::{Compact, Decode, Error as CodecError};
use log::*;
use sp_core::H256;

/// A collection of events obtained from a block, bundled with the necessary
/// information needed to decode and iterate over them.
//
// In subxt, this was generic over a `Config` type, but it's sole usage was to derive the
// hash type. We omitted this here and use the `ac_primitives::Hash` instead.
#[derive(Clone, Debug)]
pub struct Events<Hash> {
	metadata: Metadata,
	block_hash: Hash,
	// Note; raw event bytes are prefixed with a Compact<u32> containing
	// the number of events to be decoded. The start_idx reflects that, so
	// that we can skip over those bytes when decoding them
	event_bytes: Arc<[u8]>,
	start_idx: usize,
	num_events: u32,
}

impl<Hash: Copy> Events<Hash> {
	pub fn new(metadata: Metadata, block_hash: Hash, event_bytes: Vec<u8>) -> Self {
		// event_bytes is a SCALE encoded vector of events. So, pluck the
		// compact encoded length from the front, leaving the remaining bytes
		// for our iterating to decode.
		//
		// Note: if we get no bytes back, avoid an error reading vec length
		// and default to 0 events.
		let cursor = &mut &*event_bytes;
		let num_events = <Compact<u32>>::decode(cursor).unwrap_or(Compact(0)).0;

		// Start decoding after the compact encoded bytes.
		let start_idx = event_bytes.len() - cursor.len();

		Self { metadata, block_hash, event_bytes: event_bytes.into(), start_idx, num_events }
	}

	/// The number of events.
	pub fn len(&self) -> u32 {
		self.num_events
	}

	/// Are there no events in this block?
	// Note: mainly here to satisfy clippy.
	pub fn is_empty(&self) -> bool {
		self.num_events == 0
	}

	/// Return the block hash that these events are from.
	pub fn block_hash(&self) -> Hash {
		self.block_hash
	}

	/// Return the encoded bytes of the Events.
	pub fn event_bytes(&self) -> Arc<[u8]> {
		self.event_bytes.clone()
	}

	/// Iterate over all of the events, using metadata to dynamically
	/// decode them as we go, and returning the raw bytes and other associated
	/// details. If an error occurs, all subsequent iterations return `None`.
	// Dev note: The returned iterator is 'static + Send so that we can box it up and make
	// use of it with our `FilterEvents` stuff.
	pub fn iter(
		&self,
	) -> impl Iterator<Item = Result<EventDetails, Error>> + Send + Sync + 'static {
		// The event bytes ignoring the compact encoded length on the front:
		let event_bytes = self.event_bytes.clone();
		let metadata = self.metadata.clone();
		let num_events = self.num_events;

		let mut pos = self.start_idx;
		let mut index = 0;
		core::iter::from_fn(move || {
			if event_bytes.len() <= pos || num_events == index {
				None
			} else {
				match EventDetails::decode_from(metadata.clone(), event_bytes.clone(), pos, index) {
					Ok(event_details) => {
						// Skip over decoded bytes in next iteration:
						pos += event_details.bytes().len();
						// Increment the index:
						index += 1;
						// Return the event details:
						Some(Ok(event_details))
					},
					Err(e) => {
						// By setting the position to the "end" of the event bytes,
						// the cursor len will become 0 and the iterator will return `None`
						// from now on:
						pos = event_bytes.len();
						Some(Err(e))
					},
				}
			}
		})
	}

	/// Iterate through the events using metadata to dynamically decode and skip
	/// them, and return only those which should decode to the provided `Ev` type.
	/// If an error occurs, all subsequent iterations return `None`.
	pub fn find<Ev: StaticEvent>(&self) -> impl Iterator<Item = Result<Ev, Error>> + '_ {
		self.iter()
			.filter_map(|ev| ev.and_then(|ev| ev.as_event::<Ev>().map_err(Into::into)).transpose())
	}

	/// Iterate through the events using metadata to dynamically decode and skip
	/// them, and return the first event found which decodes to the provided `Ev` type.
	pub fn find_first<Ev: StaticEvent>(&self) -> Result<Option<Ev>, Error> {
		self.find::<Ev>().next().transpose()
	}

	/// Find an event that decodes to the type provided. Returns true if it was found.
	pub fn has<Ev: StaticEvent>(&self) -> Result<bool, Error> {
		Ok(self.find::<Ev>().next().transpose()?.is_some())
	}
}

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
	fn decode_from(
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		decoder::Value,
		test_utils::{event_record, events, events_raw, metadata},
	};
	use codec::Encode;
	use scale_info::TypeInfo;

	/// [`RawEventDetails`] can be annoying to test, because it contains
	/// type info in the decoded field Values. Strip that here so that
	/// we can compare fields more easily.
	#[derive(Debug, PartialEq, Clone)]
	pub struct TestRawEventDetails {
		pub phase: Phase,
		pub index: u32,
		pub pallet: String,
		pub pallet_index: u8,
		pub variant: String,
		pub variant_index: u8,
		pub fields: Vec<Value>,
	}

	/// Compare some actual [`RawEventDetails`] with a hand-constructed
	/// (probably) [`TestRawEventDetails`].
	pub fn assert_raw_events_match(
		// Just for convenience, pass in the metadata type constructed
		// by the `metadata` function above to simplify caller code.
		metadata: &Metadata,
		actual: EventDetails,
		expected: TestRawEventDetails,
	) {
		let types = &metadata.runtime_metadata().types;

		// Make sure that the bytes handed back line up with the fields handed back;
		// encode the fields back into bytes and they should be equal.
		let actual_fields = actual.field_values().expect("can decode field values (1)");
		let mut actual_bytes = vec![];
		for field in actual_fields.into_values() {
			crate::decoder::encode_as_type(field.clone(), field.context, types, &mut actual_bytes)
				.expect("should be able to encode properly");
		}
		assert_eq!(actual_bytes, actual.field_bytes());

		let actual_fields_no_context: Vec<_> = actual
			.field_values()
			.expect("can decode field values (2)")
			.into_values()
			.map(|value| value.remove_context())
			.collect();

		// Check each of the other fields:
		assert_eq!(actual.phase(), expected.phase);
		assert_eq!(actual.index(), expected.index);
		assert_eq!(actual.pallet_name(), expected.pallet);
		assert_eq!(actual.pallet_index(), expected.pallet_index);
		assert_eq!(actual.variant_name(), expected.variant);
		assert_eq!(actual.variant_index(), expected.variant_index);
		assert_eq!(actual_fields_no_context, expected.fields);
	}

	#[test]
	fn dynamically_decode_single_event() {
		#[derive(Clone, Debug, PartialEq, Decode, Encode, TypeInfo)]
		enum Event {
			A(u8, bool, Vec<String>),
		}

		// Create fake metadata that knows about our single event, above:
		let metadata = metadata::<Event>();

		// Encode our events in the format we expect back from a node, and
		// construst an Events object to iterate them:
		let event = Event::A(1, true, vec!["Hi".into()]);
		let events = events::<Event>(
			metadata.clone(),
			vec![event_record(Phase::ApplyExtrinsic(123), event)],
		);

		let mut event_details = events.iter();
		assert_raw_events_match(
			&metadata,
			event_details.next().unwrap().unwrap(),
			TestRawEventDetails {
				phase: Phase::ApplyExtrinsic(123),
				index: 0,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![
					Value::uint(1u8),
					Value::bool(true),
					Value::unnamed_composite(vec![Value::string("Hi")]),
				],
			},
		);
		assert!(event_details.next().is_none());
	}

	#[test]
	fn dynamically_decode_multiple_events() {
		#[derive(Clone, Copy, Debug, PartialEq, Decode, Encode, TypeInfo)]
		enum Event {
			A(u8),
			B(bool),
		}

		// Create fake metadata that knows about our single event, above:
		let metadata = metadata::<Event>();

		// Encode our events in the format we expect back from a node, and
		// construst an Events object to iterate them:
		let event1 = Event::A(1);
		let event2 = Event::B(true);
		let event3 = Event::A(234);

		let events = events::<Event>(
			metadata.clone(),
			vec![
				event_record(Phase::Initialization, event1),
				event_record(Phase::ApplyExtrinsic(123), event2),
				event_record(Phase::Finalization, event3),
			],
		);

		let mut event_details = events.iter();

		assert_raw_events_match(
			&metadata,
			event_details.next().unwrap().unwrap(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Initialization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::uint(1u8)],
			},
		);
		assert_raw_events_match(
			&metadata,
			event_details.next().unwrap().unwrap(),
			TestRawEventDetails {
				index: 1,
				phase: Phase::ApplyExtrinsic(123),
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "B".to_string(),
				variant_index: 1,
				fields: vec![Value::bool(true)],
			},
		);
		assert_raw_events_match(
			&metadata,
			event_details.next().unwrap().unwrap(),
			TestRawEventDetails {
				index: 2,
				phase: Phase::Finalization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::uint(234u16)],
			},
		);
		assert!(event_details.next().is_none());
	}

	#[test]
	fn dynamically_decode_multiple_events_until_error() {
		#[derive(Clone, Debug, PartialEq, Decode, Encode, TypeInfo)]
		enum Event {
			A(u8),
			B(bool),
		}

		// Create fake metadata that knows about our single event, above:
		let metadata = metadata::<Event>();

		// Encode 2 events:
		let mut event_bytes = vec![];
		event_record(Phase::Initialization, Event::A(1)).encode_to(&mut event_bytes);
		event_record(Phase::ApplyExtrinsic(123), Event::B(true)).encode_to(&mut event_bytes);

		// Push a few naff bytes to the end (a broken third event):
		event_bytes.extend_from_slice(&[3, 127, 45, 0, 2]);

		// Encode our events in the format we expect back from a node, and
		// construst an Events object to iterate them:
		let events = events_raw(
			metadata.clone(),
			event_bytes,
			3, // 2 "good" events, and then it'll hit the naff bytes.
		);

		let mut events_iter = events.iter();
		assert_raw_events_match(
			&metadata,
			events_iter.next().unwrap().unwrap(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Initialization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::uint(1u8)],
			},
		);
		assert_raw_events_match(
			&metadata,
			events_iter.next().unwrap().unwrap(),
			TestRawEventDetails {
				index: 1,
				phase: Phase::ApplyExtrinsic(123),
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "B".to_string(),
				variant_index: 1,
				fields: vec![Value::bool(true)],
			},
		);

		// We'll hit an error trying to decode the third event:
		assert!(events_iter.next().unwrap().is_err());
		// ... and then "None" from then on.
		assert!(events_iter.next().is_none());
		assert!(events_iter.next().is_none());
	}

	#[test]
	fn compact_event_field() {
		#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
		enum Event {
			A(#[codec(compact)] u32),
		}

		// Create fake metadata that knows about our single event, above:
		let metadata = metadata::<Event>();

		// Encode our events in the format we expect back from a node, and
		// construst an Events object to iterate them:
		let events =
			events::<Event>(metadata.clone(), vec![event_record(Phase::Finalization, Event::A(1))]);

		// Dynamically decode:
		let mut event_details = events.iter();
		assert_raw_events_match(
			&metadata,
			event_details.next().unwrap().unwrap(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Finalization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::uint(1u8)],
			},
		);
		assert!(event_details.next().is_none());
	}

	#[test]
	fn compact_wrapper_struct_field() {
		#[derive(Clone, Decode, Debug, PartialEq, Encode, TypeInfo)]
		enum Event {
			A(#[codec(compact)] CompactWrapper),
		}

		#[derive(Clone, Decode, Debug, PartialEq, codec::CompactAs, Encode, TypeInfo)]
		struct CompactWrapper(u64);

		// Create fake metadata that knows about our single event, above:
		let metadata = metadata::<Event>();

		// Encode our events in the format we expect back from a node, and
		// construct an Events object to iterate them:
		let events = events::<Event>(
			metadata.clone(),
			vec![event_record(Phase::Finalization, Event::A(CompactWrapper(1)))],
		);

		// Dynamically decode:
		let mut event_details = events.iter();
		assert_raw_events_match(
			&metadata,
			event_details.next().unwrap().unwrap(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Finalization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::unnamed_composite(vec![Value::uint(1u8)])],
			},
		);
		assert!(event_details.next().is_none());
	}

	#[test]
	fn event_containing_explicit_index() {
		#[derive(Clone, Debug, PartialEq, Eq, Decode, Encode, TypeInfo)]
		#[repr(u8)]
		#[allow(trivial_numeric_casts, clippy::unnecessary_cast)] // required because the Encode derive produces a warning otherwise
		pub enum MyType {
			B = 10u8,
		}

		#[derive(Clone, Debug, PartialEq, Decode, Encode, TypeInfo)]
		enum Event {
			A(MyType),
		}

		// Create fake metadata that knows about our single event, above:
		let metadata = metadata::<Event>();

		// Encode our events in the format we expect back from a node, and
		// construct an Events object to iterate them:
		let events = events::<Event>(
			metadata.clone(),
			vec![event_record(Phase::Finalization, Event::A(MyType::B))],
		);

		// Dynamically decode:
		let mut event_details = events.iter();
		assert_raw_events_match(
			&metadata,
			event_details.next().unwrap().unwrap(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Finalization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::unnamed_variant("B", vec![])],
			},
		);
		assert!(event_details.next().is_none());
	}
}
