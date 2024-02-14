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
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
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

/// Event related test utilities used outside this module.
#[cfg(test)]
pub(crate) mod test_utils {
	use super::*;
	use crate::{events::Compact, Events};
	use codec::Encode;
	use frame_metadata::{
		v15::{
			CustomMetadata, ExtrinsicMetadata, OuterEnums, PalletEventMetadata, PalletMetadata,
			RuntimeMetadataV15,
		},
		RuntimeMetadataPrefixed,
	};
	use scale_info::{meta_type, TypeInfo};
	use sp_core::H256;

	/// An "outer" events enum containing exactly one event.
	#[derive(
		Encode,
		Decode,
		TypeInfo,
		Clone,
		Debug,
		PartialEq,
		Eq,
		scale_encode::EncodeAsType,
		scale_decode::DecodeAsType,
	)]
	pub enum AllEvents<Ev> {
		Test(Ev),
	}

	/// This encodes to the same format an event is expected to encode to
	/// in node System.Events storage.
	#[derive(Encode)]
	pub struct EventRecord<E: Encode> {
		phase: Phase,
		event: AllEvents<E>,
		topics: Vec<H256>,
	}

	impl<E: Encode> EventRecord<E> {
		/// Create a new event record with the given phase, event, and topics.
		pub fn new(phase: Phase, event: E, topics: Vec<H256>) -> Self {
			Self { phase, event: AllEvents::Test(event), topics }
		}
	}

	/// Build an EventRecord, which encoded events in the format expected
	/// to be handed back from storage queries to System.Events.
	pub fn event_record<E: Encode>(phase: Phase, event: E) -> EventRecord<E> {
		EventRecord::new(phase, event, vec![])
	}

	/// Build fake metadata consisting of a single pallet that knows
	/// about the event type provided.
	pub fn metadata<E: TypeInfo + 'static>() -> Metadata {
		// Extrinsic needs to contain at least the generic type parameter "Call"
		// for the metadata to be valid.
		// The "Call" type from the metadata is used to decode extrinsics.
		// In reality, the extrinsic type has "Call", "Address", "Extra", "Signature" generic types.
		#[allow(unused)]
		#[derive(TypeInfo)]
		struct ExtrinsicType<Call> {
			call: Call,
		}
		// Because this type is used to decode extrinsics, we expect this to be a TypeDefVariant.
		// Each pallet must contain one single variant.
		#[allow(unused)]
		#[derive(TypeInfo)]
		enum RuntimeCall {
			PalletName(Pallet),
		}
		// The calls of the pallet.
		#[allow(unused)]
		#[derive(TypeInfo)]
		enum Pallet {
			#[allow(unused)]
			SomeCall,
		}

		let pallets = vec![PalletMetadata {
			name: "Test",
			storage: None,
			calls: None,
			event: Some(PalletEventMetadata { ty: meta_type::<E>() }),
			constants: vec![],
			error: None,
			index: 0,
			docs: vec![],
		}];

		let extrinsic = ExtrinsicMetadata {
			version: 0,
			signed_extensions: vec![],
			address_ty: meta_type::<()>(),
			call_ty: meta_type::<RuntimeCall>(),
			signature_ty: meta_type::<()>(),
			extra_ty: meta_type::<()>(),
		};

		let meta = RuntimeMetadataV15::new(
			pallets,
			extrinsic,
			meta_type::<()>(),
			vec![],
			OuterEnums {
				call_enum_ty: meta_type::<()>(),
				event_enum_ty: meta_type::<AllEvents<E>>(),
				error_enum_ty: meta_type::<()>(),
			},
			CustomMetadata { map: Default::default() },
		);
		let runtime_metadata: RuntimeMetadataPrefixed = meta.into();
		Metadata::try_from(runtime_metadata).unwrap()
	}

	/// Build an `Events` object for test purposes, based on the details provided,
	/// and with a default block hash.
	pub fn events<E: Decode + Encode>(
		metadata: Metadata,
		event_records: Vec<EventRecord<E>>,
	) -> Events<H256> {
		let num_events = event_records.len() as u32;
		let mut event_bytes = Vec::new();
		for ev in event_records {
			ev.encode_to(&mut event_bytes);
		}
		events_raw(metadata, event_bytes, num_events)
	}

	/// Much like [`events`], but takes pre-encoded events and event count, so that we can
	/// mess with the bytes in tests if we need to.
	pub fn events_raw(metadata: Metadata, event_bytes: Vec<u8>, num_events: u32) -> Events<H256> {
		// Prepend compact encoded length to event bytes:
		let mut all_event_bytes = Compact(num_events).encode();
		all_event_bytes.extend(event_bytes);
		Events::new(metadata, H256::default(), all_event_bytes)
	}
}

#[cfg(test)]
mod tests {
	use super::{
		test_utils::{event_record, events, events_raw, EventRecord},
		*,
	};
	use codec::Encode;
	use scale_info::TypeInfo;
	use scale_value::Value;
	use sp_core::H256;

	/// Build a fake wrapped metadata.
	fn metadata<E: TypeInfo + 'static>() -> Metadata {
		test_utils::metadata::<E>()
	}

	/// [`RawEventDetails`] can be annoying to test, because it contains
	/// type info in the decoded field Values. Strip that here so that
	/// we can compare fields more easily.
	#[derive(Debug, PartialEq, Eq, Clone)]
	pub struct TestRawEventDetails {
		pub phase: Phase,
		pub index: u32,
		pub pallet: String,
		pub pallet_index: u8,
		pub variant: String,
		pub variant_index: u8,
		pub fields: Vec<scale_value::Value>,
	}

	/// Compare some actual [`RawEventDetails`] with a hand-constructed
	/// (probably) [`TestRawEventDetails`].
	pub fn assert_raw_events_match(
		actual: RawEventDetails<H256>,
		expected: TestRawEventDetails,
		metadata: &Metadata,
	) {
		let actual_fields_no_context: Vec<_> = actual
			.field_values(metadata)
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

		let mut event_details_iter = events.iter();
		assert_raw_events_match(
			event_details_iter.next().unwrap().unwrap().to_raw(),
			TestRawEventDetails {
				phase: Phase::ApplyExtrinsic(123),
				index: 0,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![
					Value::u128(1),
					Value::bool(true),
					Value::unnamed_composite(vec![Value::string("Hi")]),
				],
			},
			&metadata,
		);
		assert!(event_details_iter.next().is_none());
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

		let mut event_details_iter = events.iter();

		assert_raw_events_match(
			event_details_iter.next().unwrap().unwrap().to_raw(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Initialization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::u128(1)],
			},
			&metadata,
		);
		assert_raw_events_match(
			event_details_iter.next().unwrap().unwrap().to_raw(),
			TestRawEventDetails {
				index: 1,
				phase: Phase::ApplyExtrinsic(123),
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "B".to_string(),
				variant_index: 1,
				fields: vec![Value::bool(true)],
			},
			&metadata,
		);
		assert_raw_events_match(
			event_details_iter.next().unwrap().unwrap().to_raw(),
			TestRawEventDetails {
				index: 2,
				phase: Phase::Finalization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::u128(234)],
			},
			&metadata,
		);
		assert!(event_details_iter.next().is_none());
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
		let mut event_details_iter = events.iter();
		assert_raw_events_match(
			event_details_iter.next().unwrap().unwrap().to_raw(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Initialization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::u128(1)],
			},
			&metadata,
		);
		assert_raw_events_match(
			event_details_iter.next().unwrap().unwrap().to_raw(),
			TestRawEventDetails {
				index: 1,
				phase: Phase::ApplyExtrinsic(123),
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "B".to_string(),
				variant_index: 1,
				fields: vec![Value::bool(true)],
			},
			&metadata,
		);

		// We'll hit an error trying to decode the third event:
		assert!(event_details_iter.next().unwrap().is_err());
		// ... and then "None" from then on.
		assert!(event_details_iter.next().is_none());
		assert!(event_details_iter.next().is_none());
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
		let mut event_details_iter = events.iter();
		assert_raw_events_match(
			event_details_iter.next().unwrap().unwrap().to_raw(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Finalization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::u128(1)],
			},
			&metadata,
		);
		assert!(event_details_iter.next().is_none());
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
		let mut event_details_iter = events.iter();
		assert_raw_events_match(
			event_details_iter.next().unwrap().unwrap().to_raw(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Finalization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::unnamed_composite(vec![Value::u128(1)])],
			},
			&metadata,
		);
		assert!(event_details_iter.next().is_none());
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
		let mut event_details_iter = events.iter();
		assert_raw_events_match(
			event_details_iter.next().unwrap().unwrap().to_raw(),
			TestRawEventDetails {
				index: 0,
				phase: Phase::Finalization,
				pallet: "Test".to_string(),
				pallet_index: 0,
				variant: "A".to_string(),
				variant_index: 0,
				fields: vec![Value::unnamed_variant("B", vec![])],
			},
			&metadata,
		);
		assert!(event_details_iter.next().is_none());
	}

	#[test]
	fn topics() {
		#[derive(Clone, Debug, PartialEq, Decode, Encode, TypeInfo, scale_decode::DecodeAsType)]
		enum Event {
			A(u8, bool, Vec<String>),
		}

		// Create fake metadata that knows about our single event, above:
		let metadata = metadata::<Event>();

		// Encode our events in the format we expect back from a node, and
		// construct an Events object to iterate them:
		let event = Event::A(1, true, vec!["Hi".into()]);
		let topics = vec![H256::from_low_u64_le(123), H256::from_low_u64_le(456)];
		let events = events::<Event>(
			metadata,
			vec![EventRecord::new(Phase::ApplyExtrinsic(123), event, topics.clone())],
		);

		let ev = events
			.iter()
			.next()
			.expect("one event expected")
			.expect("event should be extracted OK");

		assert_eq!(topics, ev.topics());
	}

	#[test]
	fn encode_decode() {
		#[derive(Clone, Debug, PartialEq, Decode, Encode, TypeInfo, scale_decode::DecodeAsType)]
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
		let raw_event_details = events.iter().next().unwrap().unwrap().to_raw();

		// Statically Encode/Decode:
		let encoded = raw_event_details.encode();
		let decoded = RawEventDetails::decode(&mut encoded.as_slice()).unwrap();

		assert_eq!(raw_event_details, decoded);
	}
}
