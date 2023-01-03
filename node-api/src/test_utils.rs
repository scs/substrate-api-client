// This file was taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

use crate::{Events, Metadata, Phase};
/// Event related test utilities used outside this module.
use codec::Encode;
use codec::{Compact, Decode};
use frame_metadata::{
	v14::{ExtrinsicMetadata, PalletEventMetadata, PalletMetadata, RuntimeMetadataV14},
	RuntimeMetadataPrefixed,
};
use scale_info::{meta_type, TypeInfo};
use sp_core::H256;

/// An "outer" events enum containing exactly one event.
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
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

/// Build an EventRecord, which encoded events in the format expected
/// to be handed back from storage queries to System.Events.
pub fn event_record<E: Encode>(phase: Phase, event: E) -> EventRecord<E> {
	EventRecord { phase, event: AllEvents::Test(event), topics: vec![] }
}

/// Build fake metadata consisting of a single pallet that knows
/// about the event type provided.
pub fn metadata<E: TypeInfo + 'static>() -> Metadata {
	let pallets = vec![PalletMetadata {
		name: "Test",
		storage: None,
		calls: None,
		event: Some(PalletEventMetadata { ty: meta_type::<E>() }),
		constants: vec![],
		error: None,
		index: 0,
	}];

	let extrinsic =
		ExtrinsicMetadata { ty: meta_type::<()>(), version: 0, signed_extensions: vec![] };

	let v14 = RuntimeMetadataV14::new(pallets, extrinsic, meta_type::<()>());
	let runtime_metadata: RuntimeMetadataPrefixed = v14.into();

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
