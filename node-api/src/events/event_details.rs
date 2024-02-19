// This file bases on subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG.

// Copyright 2019-2023 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! A representation of a block of events.

use crate::{
	error::{DispatchError, Error},
	events::{EventMetadataDetails, RawEventDetails, RootEvent},
	Metadata, Phase, StaticEvent,
};
use alloc::sync::Arc;
use codec::{Decode, Encode};
use scale_value::{scale::TypeId, Composite};

/// The event details with the associated metadata.
// Based on subxt EventDetails.
// https://github.com/paritytech/subxt/blob/8413c4d2dd625335b9200dc2289670accdf3391a/subxt/src/events/events_type.rs#L197-L216
#[derive(Debug, Clone, Encode, Decode)]
pub struct EventDetails<Hash: Encode + Decode> {
	inner: RawEventDetails<Hash>,
	metadata: Metadata,
}

impl<Hash: Encode + Decode> EventDetails<Hash> {
	// Attempt to dynamically decode a single event from our events input.
	pub(crate) fn decode_from(
		metadata: Metadata,
		all_bytes: Arc<[u8]>,
		start_idx: usize,
		index: u32,
	) -> Result<Self, Error> {
		let inner = RawEventDetails::decode_from(&metadata, all_bytes, start_idx, index)?;
		Ok(EventDetails { inner, metadata })
	}

	/// When was the event produced?
	pub fn phase(&self) -> Phase {
		self.inner.phase()
	}

	/// What index is this event in the stored events for this block.
	pub fn index(&self) -> u32 {
		self.inner.index()
	}

	/// The index of the pallet that the event originated from.
	pub fn pallet_index(&self) -> u8 {
		// Note: never panics; we expect these bytes to exist
		// in order that the EventDetails could be created.
		self.inner.pallet_index()
	}

	/// The index of the event variant that the event originated from.
	pub fn variant_index(&self) -> u8 {
		// Note: never panics; we expect these bytes to exist
		// in order that the EventDetails could be created.
		self.inner.variant_index()
	}

	/// The name of the pallet from whence the Event originated.
	pub fn pallet_name(&self) -> &str {
		self.inner.pallet_name()
	}

	/// The name of the event (ie the name of the variant that it corresponds to).
	pub fn variant_name(&self) -> &str {
		self.inner.variant_name()
	}

	/// Fetch details from the metadata for this event.
	pub fn event_metadata(&self) -> EventMetadataDetails {
		self.inner.event_metadata_unchecked(&self.metadata)
	}

	/// Return _all_ of the bytes representing this event, which include, in order:
	/// - The phase.
	/// - Pallet and event index.
	/// - Event fields.
	/// - Event Topics.
	pub fn bytes(&self) -> &[u8] {
		self.inner.bytes()
	}

	/// Return the bytes representing the fields stored in this event.
	pub fn field_bytes(&self) -> &[u8] {
		self.inner.field_bytes()
	}

	/// Decode and provide the event fields back in the form of a [`scale_value::Composite`]
	/// type which represents the named or unnamed fields that were present in the event.
	pub fn field_values(&self) -> Result<Composite<TypeId>, Error> {
		self.inner.field_values_unchecked(&self.metadata)
	}

	/// Attempt to decode these [`EventDetails`] into a specific static event.
	/// This targets the fields within the event directly. You can also attempt to
	/// decode the entirety of the event type (including the pallet and event
	/// variants) using [`EventDetails::as_root_event()`].
	pub fn as_event<E: StaticEvent>(&self) -> Result<Option<E>, Error> {
		self.inner.as_event()
	}

	/// Attempt to decode these [`EventDetails`] into a root event type (which includes
	/// the pallet and event enum variants as well as the event fields). A compatible
	/// type for this is exposed via static codegen as a root level `Event` type.
	pub fn as_root_event<E: RootEvent>(&self) -> Result<E, Error> {
		self.inner.as_root_event_unchecked(&self.metadata)
	}

	/// Return the topics associated with this event.
	pub fn topics(&self) -> &[Hash] {
		self.inner.topics()
	}

	/// Consume original struct and return only the raw portion without metadata.
	pub fn to_raw(self) -> RawEventDetails<Hash> {
		self.inner
	}
}

impl<Hash: Encode + Decode> EventDetails<Hash> {
	/// Checks if the extrinsic has failed.
	pub fn has_failed(&self) -> bool {
		self.inner.has_failed()
	}

	/// Returns the dispatch error of the failed extrinsic, if it has failed.

	pub fn get_associated_dispatch_error(&self) -> Option<DispatchError> {
		self.inner.get_associated_dispatch_error(&self.metadata)
	}

	/// Checks if the event represents a code update (runtime update).
	pub fn is_code_update(&self) -> bool {
		self.inner.is_code_update()
	}
}
