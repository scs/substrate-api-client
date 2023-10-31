// This file was taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! Event related test utilities used outside this module.

use crate::{Events, Metadata, Phase};
use codec::{Compact, Decode, Encode};
use frame_metadata::{
	v14::{
		ExtrinsicMetadata as ExtrinsicMetadataV14, PalletEventMetadata as PalletEventMetadataV14,
		PalletMetadata as PalletMetadataV14, RuntimeMetadataV14,
	},
	v15::{
		CustomMetadata, ExtrinsicMetadata as ExtrinsicMetadataV15, OuterEnums,
		PalletEventMetadata as PalletEventMetadataV15, PalletMetadata as PalletMetadataV15,
		RuntimeMetadataV15,
	},
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

pub enum SupportedMetadataVersions {
	V14,
	V15,
}

/// Build an EventRecord, which encoded events in the format expected
/// to be handed back from storage queries to System.Events.
pub fn event_record<E: Encode>(phase: Phase, event: E) -> EventRecord<E> {
	EventRecord { phase, event: AllEvents::Test(event), topics: vec![] }
}

/// Build fake metadata consisting of a single pallet that knows
/// about the event type provided.
pub fn metadata<E: TypeInfo + 'static>() -> Metadata {
	metadata_with_version::<E>(SupportedMetadataVersions::V14)
}

/// Build fake metadata consisting of a single pallet that knows
/// about the event type provided.
pub fn metadata_with_version<E: TypeInfo + 'static>(
	version: SupportedMetadataVersions,
) -> Metadata {
	let runtime_metadata: RuntimeMetadataPrefixed = match version {
		SupportedMetadataVersions::V14 => create_dummy_runtime_v14::<E>().into(),
		SupportedMetadataVersions::V15 => {
			let pallets = vec![PalletMetadataV15 {
				name: "Test",
				storage: None,
				calls: None,
				event: Some(PalletEventMetadataV15 { ty: meta_type::<E>() }),
				constants: vec![],
				error: None,
				index: 0,
				docs: vec![],
			}];

			let extrinsic = ExtrinsicMetadataV15 {
				version: 0,
				address_ty: meta_type::<()>(),
				call_ty: meta_type::<()>(),
				signature_ty: meta_type::<()>(),
				extra_ty: meta_type::<()>(),
				signed_extensions: vec![],
			};
			let outer_enums = OuterEnums {
				call_enum_ty: meta_type::<()>(),
				event_enum_ty: meta_type::<()>(),
				error_enum_ty: meta_type::<()>(),
			};
			let custom = CustomMetadata { map: Default::default() };
			let v15 = RuntimeMetadataV15::new(
				pallets,
				extrinsic,
				meta_type::<()>(),
				vec![],
				outer_enums,
				custom,
			);
			v15.into()
		},
	};

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

fn create_dummy_runtime_v14<E: TypeInfo + 'static>() -> RuntimeMetadataV14 {
	let pallets = vec![PalletMetadataV14 {
		name: "Test",
		storage: None,
		calls: None,
		event: Some(PalletEventMetadataV14 { ty: meta_type::<E>() }),
		constants: vec![],
		error: None,
		index: 0,
	}];

	let extrinsic =
		ExtrinsicMetadataV14 { ty: meta_type::<()>(), version: 0, signed_extensions: vec![] };

	let mut v14 = RuntimeMetadataV14::new(pallets, extrinsic, meta_type::<()>());

	// Register types that are needed for v14 -> v15 conversion.
	let extrinsic_type = scale_info::Type {
		path: scale_info::Path {
			segments: vec![
				"primitives".to_string(),
				"runtime".to_string(),
				"generic".to_string(),
				"UncheckedExtrinsic".to_string(),
			],
		},
		type_params: vec![
			scale_info::TypeParameter::<scale_info::form::PortableForm> {
				name: "Address".to_string(),
				ty: Some(0.into()),
			},
			scale_info::TypeParameter::<scale_info::form::PortableForm> {
				name: "Call".to_string(),
				ty: Some(0.into()),
			},
			scale_info::TypeParameter::<scale_info::form::PortableForm> {
				name: "Signature".to_string(),
				ty: Some(0.into()),
			},
			scale_info::TypeParameter::<scale_info::form::PortableForm> {
				name: "Extra".to_string(),
				ty: Some(0.into()),
			},
		],
		type_def: scale_info::TypeDef::Composite(scale_info::TypeDefComposite { fields: vec![] }),
		docs: vec![],
	};
	let new_type_id = v14.types.types.len() as u32;
	v14.types
		.types
		.push(scale_info::PortableType { id: new_type_id, ty: extrinsic_type });
	v14.extrinsic.ty = new_type_id.into();

	let runtime_call_type = scale_info::Type {
		path: scale_info::Path { segments: vec!["RuntimeError".to_string()] },
		type_params: vec![],
		type_def: scale_info::TypeDef::Variant(scale_info::TypeDefVariant { variants: vec![] }),
		docs: vec![],
	};
	v14.types
		.types
		.push(scale_info::PortableType { id: v14.types.types.len() as u32, ty: runtime_call_type });

	let runtime_call_type = scale_info::Type {
		path: scale_info::Path { segments: vec!["RuntimeCall".to_string()] },
		type_params: vec![],
		type_def: scale_info::TypeDef::Variant(scale_info::TypeDefVariant { variants: vec![] }),
		docs: vec![],
	};
	v14.types
		.types
		.push(scale_info::PortableType { id: v14.types.types.len() as u32, ty: runtime_call_type });

	let runtime_call_type = scale_info::Type {
		path: scale_info::Path { segments: vec!["RuntimeEvent".to_string()] },
		type_params: vec![],
		type_def: scale_info::TypeDef::Variant(scale_info::TypeDefVariant { variants: vec![] }),
		docs: vec![],
	};
	v14.types
		.types
		.push(scale_info::PortableType { id: v14.types.types.len() as u32, ty: runtime_call_type });

	v14
}
