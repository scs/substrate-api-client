// This file was taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! A representation of a block of events.
//! This file bases on https://github.com/paritytech/subxt/blob/8413c4d2dd625335b9200dc2289670accdf3391a/subxt/src/events/events_type.rs#L19-L196

use crate::{error::Error, metadata::PalletMetadata, Metadata, StaticEvent};
use alloc::{sync::Arc, vec::Vec};
use codec::{Compact, Decode, Encode};

mod event_details;
mod raw_event_details;
pub use event_details::EventDetails;
pub use raw_event_details::RawEventDetails;

/// A collection of events obtained from a block, bundled with the necessary
/// information needed to decode and iterate over them.
#[derive(Clone)]
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

// Ignore the Metadata when debug-logging events; it's big and distracting.
impl<Hash: core::fmt::Debug> core::fmt::Debug for Events<Hash> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("Events")
			.field("block_hash", &self.block_hash)
			.field("event_bytes", &self.event_bytes)
			.field("start_idx", &self.start_idx)
			.field("num_events", &self.num_events)
			.finish()
	}
}

impl<Hash: Copy + Encode + Decode> Events<Hash> {
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
	) -> impl Iterator<Item = Result<EventDetails<Hash>, Error>> + Send + Sync + 'static {
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

	/// Iterate through the events using metadata to dynamically decode and skip
	/// them, and return the last event found which decodes to the provided `Ev` type.
	pub fn find_last<Ev: StaticEvent>(&self) -> Result<Option<Ev>, Error> {
		self.find::<Ev>().last().transpose()
	}

	/// Find an event that decodes to the type provided. Returns true if it was found.
	pub fn has<Ev: StaticEvent>(&self) -> Result<bool, Error> {
		Ok(self.find::<Ev>().next().transpose()?.is_some())
	}
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

/// Details for the given event plucked from the metadata.
// Based on https://github.com/paritytech/subxt/blob/8413c4d2dd625335b9200dc2289670accdf3391a/subxt/src/events/events_type.rs#L411-L415
#[derive(Clone)]
pub struct EventMetadataDetails<'a> {
	pub pallet: PalletMetadata<'a>,
	pub variant: &'a scale_info::Variant<scale_info::form::PortableForm>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		test_utils::{
			event_record, events, events_raw, metadata_with_version, SupportedMetadataVersions,
		},
		Phase,
	};
	use codec::Encode;
	use scale_info::TypeInfo;
	use scale_value::Value;
	use sp_core::H256;
	use test_case::test_case;

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
	pub fn assert_raw_events_match(actual: EventDetails<H256>, expected: TestRawEventDetails) {
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

	#[test_case(SupportedMetadataVersions::V14)]
	#[test_case(SupportedMetadataVersions::V15)]
	fn dynamically_decode_single_event(metadata_version: SupportedMetadataVersions) {
		#[derive(Clone, Debug, PartialEq, Decode, Encode, TypeInfo)]
		enum Event {
			A(u8, bool, Vec<String>),
		}

		// Create fake metadata that knows about our single event, above:
		let metadata = metadata_with_version::<Event>(metadata_version);

		// Encode our events in the format we expect back from a node, and
		// construst an Events object to iterate them:
		let event = Event::A(1, true, vec!["Hi".into()]);
		let events = events::<Event>(
			metadata.clone(),
			vec![event_record(Phase::ApplyExtrinsic(123), event)],
		);

		let mut event_details = events.iter();
		assert_raw_events_match(
			event_details.next().unwrap().unwrap(),
			TestRawEventDetails {
				phase: Phase::ApplyExtrinsic(123),
				index: 0,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![
					Value::u128(1u128),
					Value::bool(true),
					Value::unnamed_composite(vec![Value::string("Hi")]),
				],
			},
		);
		assert!(event_details.next().is_none());
	}

	#[test_case(SupportedMetadataVersions::V14)]
	#[test_case(SupportedMetadataVersions::V15)]
	fn dynamically_decode_multiple_events(metadata_version: SupportedMetadataVersions) {
		#[derive(Clone, Copy, Debug, PartialEq, Decode, Encode, TypeInfo)]
		enum Event {
			A(u8),
			B(bool),
		}

		// Create fake metadata that knows about our single event, above:
		let metadata = metadata_with_version::<Event>(metadata_version);

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
			event_details.next().unwrap().unwrap(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Initialization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::u128(1u128)],
			},
		);
		assert_raw_events_match(
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
			event_details.next().unwrap().unwrap(),
			TestRawEventDetails {
				index: 2,
				phase: Phase::Finalization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::u128(234u128)],
			},
		);
		assert!(event_details.next().is_none());
	}

	#[test_case(SupportedMetadataVersions::V14)]
	#[test_case(SupportedMetadataVersions::V15)]
	fn dynamically_decode_multiple_events_until_error(metadata_version: SupportedMetadataVersions) {
		#[derive(Clone, Debug, PartialEq, Decode, Encode, TypeInfo)]
		enum Event {
			A(u8),
			B(bool),
		}

		// Create fake metadata that knows about our single event, above:
		let metadata = metadata_with_version::<Event>(metadata_version);

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
			events_iter.next().unwrap().unwrap(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Initialization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::u128(1u128)],
			},
		);
		assert_raw_events_match(
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

	#[test_case(SupportedMetadataVersions::V14)]
	#[test_case(SupportedMetadataVersions::V15)]
	fn compact_event_field(metadata_version: SupportedMetadataVersions) {
		#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo)]
		enum Event {
			A(#[codec(compact)] u32),
		}

		// Create fake metadata that knows about our single event, above:
		let metadata = metadata_with_version::<Event>(metadata_version);

		// Encode our events in the format we expect back from a node, and
		// construst an Events object to iterate them:
		let events =
			events::<Event>(metadata.clone(), vec![event_record(Phase::Finalization, Event::A(1))]);

		// Dynamically decode:
		let mut event_details = events.iter();
		assert_raw_events_match(
			event_details.next().unwrap().unwrap(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Finalization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::u128(1u128)],
			},
		);
		assert!(event_details.next().is_none());
	}

	#[test_case(SupportedMetadataVersions::V14)]
	#[test_case(SupportedMetadataVersions::V15)]
	fn compact_wrapper_struct_field(metadata_version: SupportedMetadataVersions) {
		#[derive(Clone, Decode, Debug, PartialEq, Encode, TypeInfo)]
		enum Event {
			A(#[codec(compact)] CompactWrapper),
		}

		#[derive(Clone, Decode, Debug, PartialEq, codec::CompactAs, Encode, TypeInfo)]
		struct CompactWrapper(u64);

		// Create fake metadata that knows about our single event, above:
		let metadata = metadata_with_version::<Event>(metadata_version);

		// Encode our events in the format we expect back from a node, and
		// construct an Events object to iterate them:
		let events = events::<Event>(
			metadata.clone(),
			vec![event_record(Phase::Finalization, Event::A(CompactWrapper(1)))],
		);

		// Dynamically decode:
		let mut event_details = events.iter();
		assert_raw_events_match(
			event_details.next().unwrap().unwrap(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Finalization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::unnamed_composite(vec![Value::u128(1)])],
			},
		);
		assert!(event_details.next().is_none());
	}

	#[test_case(SupportedMetadataVersions::V14)]
	#[test_case(SupportedMetadataVersions::V15)]
	fn event_containing_explicit_index(metadata_version: SupportedMetadataVersions) {
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
		let metadata = metadata_with_version::<Event>(metadata_version);

		// Encode our events in the format we expect back from a node, and
		// construct an Events object to iterate them:
		let events = events::<Event>(
			metadata.clone(),
			vec![event_record(Phase::Finalization, Event::A(MyType::B))],
		);

		// Dynamically decode:
		let mut event_details = events.iter();
		assert_raw_events_match(
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
