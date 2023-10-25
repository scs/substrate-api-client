// This file was taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG.
//
// Copyright 2019-2023 Parity Technologies (UK) Ltd and Supercomputing Systems AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

use super::error::MetadataConversionError;
use alloc::{
	collections::BTreeMap,
	format,
	string::{String, ToString},
	vec,
	vec::Vec,
};
use frame_metadata::{v14, v15};
use scale_info::TypeDef;

pub fn v14_to_v15(
	mut metadata: v14::RuntimeMetadataV14,
) -> Result<v15::RuntimeMetadataV15, MetadataConversionError> {
	// Find the extrinsic types.
	let extrinsic_parts = ExtrinsicPartTypeIds::new(&metadata)?;

	let outer_enums = generate_outer_enums(&mut metadata)?;

	Ok(v15::RuntimeMetadataV15 {
        types: metadata.types,
        pallets: metadata
            .pallets
            .into_iter()
            .map(|pallet| frame_metadata::v15::PalletMetadata {
                name: pallet.name,
                storage: pallet
                    .storage
                    .map(|storage| frame_metadata::v15::PalletStorageMetadata {
                        prefix: storage.prefix,
                        entries: storage
                            .entries
                            .into_iter()
                            .map(|entry| {
                                let modifier = match entry.modifier {
                                    frame_metadata::v14::StorageEntryModifier::Optional => {
                                        frame_metadata::v15::StorageEntryModifier::Optional
                                    }
                                    frame_metadata::v14::StorageEntryModifier::Default => {
                                        frame_metadata::v15::StorageEntryModifier::Default
                                    }
                                };

                                let ty = match entry.ty {
                                    frame_metadata::v14::StorageEntryType::Plain(ty) => {
                                        frame_metadata::v15::StorageEntryType::Plain(ty)
                                    },
                                    frame_metadata::v14::StorageEntryType::Map {
                                        hashers,
                                        key,
                                        value,
                                    } => frame_metadata::v15::StorageEntryType::Map {
                                        hashers: hashers.into_iter().map(|hasher| match hasher {
                                            frame_metadata::v14::StorageHasher::Blake2_128 => frame_metadata::v15::StorageHasher::Blake2_128,
                                            frame_metadata::v14::StorageHasher::Blake2_256 => frame_metadata::v15::StorageHasher::Blake2_256,
                                            frame_metadata::v14::StorageHasher::Blake2_128Concat  => frame_metadata::v15::StorageHasher::Blake2_128Concat ,
                                            frame_metadata::v14::StorageHasher::Twox128 => frame_metadata::v15::StorageHasher::Twox128,
                                            frame_metadata::v14::StorageHasher::Twox256 => frame_metadata::v15::StorageHasher::Twox256,
                                            frame_metadata::v14::StorageHasher::Twox64Concat => frame_metadata::v15::StorageHasher::Twox64Concat,
                                            frame_metadata::v14::StorageHasher::Identity=> frame_metadata::v15::StorageHasher::Identity,
                                        }).collect(),
                                        key,
                                        value,
                                    },
                                };

                                frame_metadata::v15::StorageEntryMetadata {
                                    name: entry.name,
                                    modifier,
                                    ty,
                                    default: entry.default,
                                    docs: entry.docs,
                                }
                            })
                            .collect(),
                    }),
                calls: pallet.calls.map(|calls| frame_metadata::v15::PalletCallMetadata { ty: calls.ty } ),
                event: pallet.event.map(|event| frame_metadata::v15::PalletEventMetadata { ty: event.ty } ),
                constants: pallet.constants.into_iter().map(|constant| frame_metadata::v15::PalletConstantMetadata {
                    name: constant.name,
                    ty: constant.ty,
                    value: constant.value,
                    docs: constant.docs,
                } ).collect(),
                error: pallet.error.map(|error| frame_metadata::v15::PalletErrorMetadata { ty: error.ty } ),
                index: pallet.index,
                docs: Default::default(),
            })
            .collect(),
        extrinsic: frame_metadata::v15::ExtrinsicMetadata {
            version: metadata.extrinsic.version,
            signed_extensions: metadata.extrinsic.signed_extensions.into_iter().map(|ext| {
                frame_metadata::v15::SignedExtensionMetadata {
                    identifier: ext.identifier,
                    ty: ext.ty,
                    additional_signed: ext.additional_signed,
                }
            }).collect(),
            address_ty: extrinsic_parts.address.into(),
            call_ty: extrinsic_parts.call.into(),
            signature_ty: extrinsic_parts.signature.into(),
            extra_ty: extrinsic_parts.extra.into(),
        },
        ty: metadata.ty,
        apis: Default::default(),
        outer_enums,
        custom: v15::CustomMetadata {
            map: Default::default(),
        },
    })
}

/// The type IDs extracted from the metadata that represent the
/// generic type parameters passed to the `UncheckedExtrinsic` from
/// the substrate-based chain.
struct ExtrinsicPartTypeIds {
	address: u32,
	call: u32,
	signature: u32,
	extra: u32,
}

impl ExtrinsicPartTypeIds {
	/// Extract the generic type parameters IDs from the extrinsic type.
	fn new(metadata: &v14::RuntimeMetadataV14) -> Result<Self, MetadataConversionError> {
		const ADDRESS: &str = "Address";
		const CALL: &str = "Call";
		const SIGNATURE: &str = "Signature";
		const EXTRA: &str = "Extra";

		let extrinsic_id = metadata.extrinsic.ty.id;
		let Some(extrinsic_ty) = metadata.types.resolve(extrinsic_id) else {
			return Err(MetadataConversionError::TypeNotFound(extrinsic_id))
		};

		let params: BTreeMap<_, _> = extrinsic_ty
			.type_params
			.iter()
			.map(|ty_param| {
				let Some(ty) = ty_param.ty else {
					return Err(MetadataConversionError::TypeNameNotFound(ty_param.name.clone()))
				};

				Ok((ty_param.name.as_str(), ty.id))
			})
			.collect::<Result<_, _>>()?;

		let Some(address) = params.get(ADDRESS) else {
			return Err(MetadataConversionError::TypeNameNotFound(ADDRESS.into()))
		};
		let Some(call) = params.get(CALL) else {
			return Err(MetadataConversionError::TypeNameNotFound(CALL.into()))
		};
		let Some(signature) = params.get(SIGNATURE) else {
			return Err(MetadataConversionError::TypeNameNotFound(SIGNATURE.into()))
		};
		let Some(extra) = params.get(EXTRA) else {
			return Err(MetadataConversionError::TypeNameNotFound(EXTRA.into()))
		};

		Ok(ExtrinsicPartTypeIds {
			address: *address,
			call: *call,
			signature: *signature,
			extra: *extra,
		})
	}
}

fn generate_outer_enums(
	metadata: &mut v14::RuntimeMetadataV14,
) -> Result<v15::OuterEnums<scale_info::form::PortableForm>, MetadataConversionError> {
	let find_type = |name: &str| {
		metadata.types.types.iter().find_map(|ty| {
			let ident = ty.ty.path.ident()?;

			if ident != name {
				return None
			}

			let TypeDef::Variant(_) = &ty.ty.type_def else { return None };

			Some((ty.id, ty.ty.path.segments.clone()))
		})
	};

	let Some((call_enum, mut call_path)) = find_type("RuntimeCall") else {
		return Err(MetadataConversionError::TypeNameNotFound("RuntimeCall".into()))
	};

	let Some((event_enum, _)) = find_type("RuntimeEvent") else {
		return Err(MetadataConversionError::TypeNameNotFound("RuntimeEvent".into()))
	};

	let error_enum = if let Some((error_enum, _)) = find_type("RuntimeError") {
		error_enum
	} else {
		let Some(last) = call_path.last_mut() else {
			return Err(MetadataConversionError::InvalidTypePath("RuntimeCall".into()))
		};
		*last = "RuntimeError".to_string();
		generate_outer_error_enum_type(metadata, call_path)
	};

	Ok(v15::OuterEnums {
		call_enum_ty: call_enum.into(),
		event_enum_ty: event_enum.into(),
		error_enum_ty: error_enum.into(),
	})
}

/// Generates an outer `RuntimeError` enum type and adds it to the metadata.
///
/// Returns the id of the generated type from the registry.
fn generate_outer_error_enum_type(
	metadata: &mut v14::RuntimeMetadataV14,
	path_segments: Vec<String>,
) -> u32 {
	let variants: Vec<_> = metadata
		.pallets
		.iter()
		.filter_map(|pallet| {
			let error = &pallet.error.clone()?;

			let path = format!("{}Error", pallet.name);
			let ty = error.ty.id.into();

			Some(scale_info::Variant {
				name: pallet.name.clone(),
				fields: vec![scale_info::Field {
					name: None,
					ty,
					type_name: Some(path),
					docs: vec![],
				}],
				index: pallet.index,
				docs: vec![],
			})
		})
		.collect();

	let enum_type = scale_info::Type {
		path: scale_info::Path { segments: path_segments },
		type_params: vec![],
		type_def: scale_info::TypeDef::Variant(scale_info::TypeDefVariant { variants }),
		docs: vec![],
	};

	let enum_type_id = metadata.types.types.len() as u32;

	metadata
		.types
		.types
		.push(scale_info::PortableType { id: enum_type_id, ty: enum_type });

	enum_type_id
}

#[cfg(test)]
mod tests {
	use super::*;
	use codec::Decode;
	use frame_metadata::{
		v14::{ExtrinsicMetadata, RuntimeMetadataV14},
		RuntimeMetadata, RuntimeMetadataPrefixed,
	};
	use scale_info::{meta_type, IntoPortable, TypeInfo};
	use sp_core::Bytes;
	use std::{fs, marker::PhantomData};

	fn load_v14_metadata() -> RuntimeMetadataV14 {
		let encoded_metadata: Bytes = fs::read("./../ksm_metadata_v14.bin").unwrap().into();
		let runtime_metadata_prefixed: RuntimeMetadataPrefixed =
			Decode::decode(&mut encoded_metadata.0.as_slice()).unwrap();

		match runtime_metadata_prefixed.1 {
			RuntimeMetadata::V14(ref metadata) => metadata.clone(),
			_ => unimplemented!(),
		}
	}

	#[test]
	fn test_extrinsic_id_generation() {
		let v14 = load_v14_metadata();

		let v15 = v14_to_v15(v14.clone()).unwrap();

		let ext_ty = v14.types.resolve(v14.extrinsic.ty.id).unwrap();
		let addr_id = ext_ty
			.type_params
			.iter()
			.find_map(|ty| if ty.name == "Address" { Some(ty.ty.unwrap().id) } else { None })
			.unwrap();
		let call_id = ext_ty
			.type_params
			.iter()
			.find_map(|ty| if ty.name == "Call" { Some(ty.ty.unwrap().id) } else { None })
			.unwrap();
		let extra_id = ext_ty
			.type_params
			.iter()
			.find_map(|ty| if ty.name == "Extra" { Some(ty.ty.unwrap().id) } else { None })
			.unwrap();
		let signature_id = ext_ty
			.type_params
			.iter()
			.find_map(|ty| if ty.name == "Signature" { Some(ty.ty.unwrap().id) } else { None })
			.unwrap();

		// Position in type registry shouldn't change.
		assert_eq!(v15.extrinsic.address_ty.id, addr_id);
		assert_eq!(v15.extrinsic.call_ty.id, call_id);
		assert_eq!(v15.extrinsic.extra_ty.id, extra_id);
		assert_eq!(v15.extrinsic.signature_ty.id, signature_id);

		let v15_addr = v15.types.resolve(v15.extrinsic.address_ty.id).unwrap();
		let v14_addr = v14.types.resolve(addr_id).unwrap();
		assert_eq!(v15_addr, v14_addr);

		let v15_call = v15.types.resolve(v15.extrinsic.call_ty.id).unwrap();
		let v14_call = v14.types.resolve(call_id).unwrap();
		assert_eq!(v15_call, v14_call);

		let v15_extra = v15.types.resolve(v15.extrinsic.extra_ty.id).unwrap();
		let v14_extra = v14.types.resolve(extra_id).unwrap();
		assert_eq!(v15_extra, v14_extra);

		let v15_sign = v15.types.resolve(v15.extrinsic.signature_ty.id).unwrap();
		let v14_sign = v14.types.resolve(signature_id).unwrap();
		assert_eq!(v15_sign, v14_sign);
	}

	#[test]
	fn test_missing_extrinsic_types() {
		#[derive(TypeInfo)]
		struct Runtime;

		let generate_metadata = |extrinsic_ty| {
			let mut registry = scale_info::Registry::new();

			let ty = registry.register_type(&meta_type::<Runtime>());

			let extrinsic =
				ExtrinsicMetadata { ty: extrinsic_ty, version: 0, signed_extensions: vec![] }
					.into_portable(&mut registry);

			v14::RuntimeMetadataV14 { types: registry.into(), pallets: Vec::new(), extrinsic, ty }
		};

		let metadata = generate_metadata(meta_type::<()>());
		let err = v14_to_v15(metadata).unwrap_err();
		assert_eq!(err, MetadataConversionError::TypeNameNotFound("Address".into()));

		#[derive(TypeInfo)]
		struct ExtrinsicNoCall<Address, Signature, Extra> {
			_phantom: PhantomData<(Address, Signature, Extra)>,
		}
		let metadata = generate_metadata(meta_type::<ExtrinsicNoCall<(), (), ()>>());
		let err = v14_to_v15(metadata).unwrap_err();
		assert_eq!(err, MetadataConversionError::TypeNameNotFound("Call".into()));

		#[derive(TypeInfo)]
		struct ExtrinsicNoSign<Call, Address, Extra> {
			_phantom: PhantomData<(Call, Address, Extra)>,
		}
		let metadata = generate_metadata(meta_type::<ExtrinsicNoSign<(), (), ()>>());
		let err = v14_to_v15(metadata).unwrap_err();
		assert_eq!(err, MetadataConversionError::TypeNameNotFound("Signature".into()));

		#[derive(TypeInfo)]
		struct ExtrinsicNoExtra<Call, Address, Signature> {
			_phantom: PhantomData<(Call, Address, Signature)>,
		}
		let metadata = generate_metadata(meta_type::<ExtrinsicNoExtra<(), (), ()>>());
		let err = v14_to_v15(metadata).unwrap_err();
		assert_eq!(err, MetadataConversionError::TypeNameNotFound("Extra".into()));
	}

	#[test]
	fn test_missing_outer_enum_types() {
		#[derive(TypeInfo)]
		struct Runtime;

		#[derive(TypeInfo)]
		enum RuntimeCall {}
		#[derive(TypeInfo)]
		enum RuntimeEvent {}

		#[allow(unused)]
		#[derive(TypeInfo)]
		struct ExtrinsicType<Address, Call, Signature, Extra> {
			pub signature: Option<(Address, Signature, Extra)>,
			pub function: Call,
		}

		// Missing runtime call.
		{
			let mut registry = scale_info::Registry::new();
			let ty = registry.register_type(&meta_type::<Runtime>());
			registry.register_type(&meta_type::<RuntimeEvent>());

			let extrinsic = ExtrinsicMetadata {
				ty: meta_type::<ExtrinsicType<(), (), (), ()>>(),
				version: 0,
				signed_extensions: vec![],
			}
			.into_portable(&mut registry);

			let metadata = v14::RuntimeMetadataV14 {
				types: registry.into(),
				pallets: Vec::new(),
				extrinsic,
				ty,
			};

			let err = v14_to_v15(metadata).unwrap_err();
			assert_eq!(err, MetadataConversionError::TypeNameNotFound("RuntimeCall".into()));
		}

		// Missing runtime event.
		{
			let mut registry = scale_info::Registry::new();
			let ty = registry.register_type(&meta_type::<Runtime>());
			registry.register_type(&meta_type::<RuntimeCall>());

			let extrinsic = ExtrinsicMetadata {
				ty: meta_type::<ExtrinsicType<(), (), (), ()>>(),
				version: 0,
				signed_extensions: vec![],
			}
			.into_portable(&mut registry);

			let metadata = v14::RuntimeMetadataV14 {
				types: registry.into(),
				pallets: Vec::new(),
				extrinsic,
				ty,
			};

			let err = v14_to_v15(metadata).unwrap_err();
			assert_eq!(err, MetadataConversionError::TypeNameNotFound("RuntimeEvent".into()));
		}
	}
}
