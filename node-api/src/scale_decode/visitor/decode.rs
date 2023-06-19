// This file was taken from scale-decode (Parity Technologies (UK))
// https://github.com/paritytech/scale-decode/
// And was adapted by Supercomputing Systems AG.
//
// Copyright (C) 2022-2023 Parity Technologies (UK) Ltd. (admin@parity.io)
// This file is a part of the scale-decode crate.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//         http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::scale_decode::{
	visitor::{
		Array, BitSequence, Compact, CompactLocation, Composite, DecodeAsTypeResult, DecodeError,
		Sequence, Str, Tuple, TypeId, Variant, Visitor,
	},
	Field,
};
use codec::{self, Decode};
use scale_info::{
	form::PortableForm, Path, PortableRegistry, TypeDef, TypeDefArray, TypeDefBitSequence,
	TypeDefCompact, TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple,
	TypeDefVariant,
};

/// Decode data according to the type ID and [`PortableRegistry`] provided.
/// The provided pointer to the data slice will be moved forwards as needed
/// depending on what was decoded, and a method on the provided [`Visitor`]
/// will be called depending on the type that needs to be decoded.
pub fn decode_with_visitor<'scale, 'info, V: Visitor>(
	data: &mut &'scale [u8],
	ty_id: u32,
	types: &'info PortableRegistry,
	visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
	// Provide option to "bail out" and do something custom first.
	let visitor = match visitor.unchecked_decode_as_type(data, TypeId(ty_id), types) {
		DecodeAsTypeResult::Decoded(r) => return r,
		DecodeAsTypeResult::Skipped(v) => v,
	};

	let ty = types.resolve(ty_id).ok_or(DecodeError::TypeIdNotFound(ty_id))?;
	let ty_id = TypeId(ty_id);
	let path = &ty.path;

	match &ty.type_def {
		TypeDef::Composite(inner) =>
			decode_composite_value(data, ty_id, path, inner, types, visitor),
		TypeDef::Variant(inner) => decode_variant_value(data, ty_id, path, inner, types, visitor),
		TypeDef::Sequence(inner) => decode_sequence_value(data, ty_id, inner, types, visitor),
		TypeDef::Array(inner) => decode_array_value(data, ty_id, inner, types, visitor),
		TypeDef::Tuple(inner) => decode_tuple_value(data, ty_id, inner, types, visitor),
		TypeDef::Primitive(inner) => decode_primitive_value(data, ty_id, inner, visitor),
		TypeDef::Compact(inner) => decode_compact_value(data, ty_id, inner, types, visitor),
		TypeDef::BitSequence(inner) =>
			decode_bit_sequence_value(data, ty_id, inner, types, visitor),
	}
}

fn decode_composite_value<'scale, 'info, V: Visitor>(
	data: &mut &'scale [u8],
	ty_id: TypeId,
	path: &'info Path<PortableForm>,
	ty: &'info TypeDefComposite<PortableForm>,
	types: &'info PortableRegistry,
	visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
	let mut fields = ty.fields.iter().map(|f| Field::new(f.ty.id, f.name.as_deref()));
	let mut items = Composite::new(data, path, &mut fields, types);
	let res = visitor.visit_composite(&mut items, ty_id);

	// Skip over any bytes that the visitor chose not to decode:
	items.skip_decoding()?;
	*data = items.bytes_from_undecoded();

	res
}

fn decode_variant_value<'scale, 'info, V: Visitor>(
	data: &mut &'scale [u8],
	ty_id: TypeId,
	path: &'info Path<PortableForm>,
	ty: &'info TypeDefVariant<PortableForm>,
	types: &'info PortableRegistry,
	visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
	let mut variant = Variant::new(data, path, ty, types)?;
	let res = visitor.visit_variant(&mut variant, ty_id);

	// Skip over any bytes that the visitor chose not to decode:
	variant.skip_decoding()?;
	*data = variant.bytes_from_undecoded();

	res
}

fn decode_sequence_value<'scale, 'info, V: Visitor>(
	data: &mut &'scale [u8],
	ty_id: TypeId,
	ty: &'info TypeDefSequence<PortableForm>,
	types: &'info PortableRegistry,
	visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
	let mut items = Sequence::new(data, ty.type_param.id, types)?;
	let res = visitor.visit_sequence(&mut items, ty_id);

	// Skip over any bytes that the visitor chose not to decode:
	items.skip_decoding()?;
	*data = items.bytes_from_undecoded();

	res
}

fn decode_array_value<'scale, 'info, V: Visitor>(
	data: &mut &'scale [u8],
	ty_id: TypeId,
	ty: &'info TypeDefArray<PortableForm>,
	types: &'info PortableRegistry,
	visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
	let len = ty.len as usize;
	let mut arr = Array::new(data, ty.type_param.id, len, types);
	let res = visitor.visit_array(&mut arr, ty_id);

	// Skip over any bytes that the visitor chose not to decode:
	arr.skip_decoding()?;
	*data = arr.bytes_from_undecoded();

	res
}

fn decode_tuple_value<'scale, 'info, V: Visitor>(
	data: &mut &'scale [u8],
	ty_id: TypeId,
	ty: &'info TypeDefTuple<PortableForm>,
	types: &'info PortableRegistry,
	visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
	let mut fields = ty.fields.iter().map(|f| Field::unnamed(f.id));
	let mut items = Tuple::new(data, &mut fields, types);
	let res = visitor.visit_tuple(&mut items, ty_id);

	// Skip over any bytes that the visitor chose not to decode:
	items.skip_decoding()?;
	*data = items.bytes_from_undecoded();

	res
}

fn decode_primitive_value<'scale, 'info, V: Visitor>(
	data: &mut &'scale [u8],
	ty_id: TypeId,
	ty: &'info TypeDefPrimitive,
	visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
	fn decode_32_bytes<'scale>(data: &mut &'scale [u8]) -> Result<&'scale [u8; 32], DecodeError> {
		// Pull an array from the data if we can, preserving the lifetime.
		let arr: &'scale [u8; 32] = match (*data).try_into() {
			Ok(arr) => arr,
			Err(_) => return Err(DecodeError::NotEnoughInput),
		};
		// If this works out, remember to shift data 32 bytes forward.
		*data = &data[32..];
		Ok(arr)
	}

	match ty {
		TypeDefPrimitive::Bool => {
			let b = bool::decode(data).map_err(|e| e.into())?;
			visitor.visit_bool(b, ty_id)
		},
		TypeDefPrimitive::Char => {
			// Treat chars as u32's
			let val = u32::decode(data).map_err(|e| e.into())?;
			let c = char::from_u32(val).ok_or(DecodeError::InvalidChar(val))?;
			visitor.visit_char(c, ty_id)
		},
		TypeDefPrimitive::Str => {
			// Avoid allocating; don't decode into a String. instead, pull the bytes
			// and let the visitor decide whether to use them or not.
			let mut s = Str::new(data)?;
			// Since we aren't decoding here, shift our bytes along to after the str:
			*data = s.bytes_after();
			visitor.visit_str(&mut s, ty_id)
		},
		TypeDefPrimitive::U8 => {
			let n = u8::decode(data).map_err(|e| e.into())?;
			visitor.visit_u8(n, ty_id)
		},
		TypeDefPrimitive::U16 => {
			let n = u16::decode(data).map_err(|e| e.into())?;
			visitor.visit_u16(n, ty_id)
		},
		TypeDefPrimitive::U32 => {
			let n = u32::decode(data).map_err(|e| e.into())?;
			visitor.visit_u32(n, ty_id)
		},
		TypeDefPrimitive::U64 => {
			let n = u64::decode(data).map_err(|e| e.into())?;
			visitor.visit_u64(n, ty_id)
		},
		TypeDefPrimitive::U128 => {
			let n = u128::decode(data).map_err(|e| e.into())?;
			visitor.visit_u128(n, ty_id)
		},
		TypeDefPrimitive::U256 => {
			let arr = decode_32_bytes(data)?;
			visitor.visit_u256(arr, ty_id)
		},
		TypeDefPrimitive::I8 => {
			let n = i8::decode(data).map_err(|e| e.into())?;
			visitor.visit_i8(n, ty_id)
		},
		TypeDefPrimitive::I16 => {
			let n = i16::decode(data).map_err(|e| e.into())?;
			visitor.visit_i16(n, ty_id)
		},
		TypeDefPrimitive::I32 => {
			let n = i32::decode(data).map_err(|e| e.into())?;
			visitor.visit_i32(n, ty_id)
		},
		TypeDefPrimitive::I64 => {
			let n = i64::decode(data).map_err(|e| e.into())?;
			visitor.visit_i64(n, ty_id)
		},
		TypeDefPrimitive::I128 => {
			let n = i128::decode(data).map_err(|e| e.into())?;
			visitor.visit_i128(n, ty_id)
		},
		TypeDefPrimitive::I256 => {
			let arr = decode_32_bytes(data)?;
			visitor.visit_i256(arr, ty_id)
		},
	}
}

fn decode_compact_value<'scale, 'info, V: Visitor>(
	data: &mut &'scale [u8],
	ty_id: TypeId,
	ty: &'info TypeDefCompact<PortableForm>,
	types: &'info PortableRegistry,
	visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
	#[allow(clippy::too_many_arguments)]
	fn decode_compact<'scale, 'info, V: Visitor>(
		data: &mut &'scale [u8],
		outermost_ty_id: TypeId,
		current_type_id: TypeId,
		mut locations: smallvec::SmallVec<[CompactLocation<'info>; 8]>,
		inner: &'info scale_info::Type<PortableForm>,
		types: &'info PortableRegistry,
		visitor: V,
	) -> Result<V::Value<'scale, 'info>, V::Error> {
		use TypeDefPrimitive::*;
		match &inner.type_def {
			// It's obvious how to decode basic primitive unsigned types, since we have impls for them.
			TypeDef::Primitive(U8) => {
				locations.push(CompactLocation::Primitive(current_type_id));
				let n = codec::Compact::<u8>::decode(data).map_err(|e| e.into())?.0;
				let c = Compact::new(n, locations.as_slice());
				visitor.visit_compact_u8(c, outermost_ty_id)
			},
			TypeDef::Primitive(U16) => {
				locations.push(CompactLocation::Primitive(current_type_id));
				let n = codec::Compact::<u16>::decode(data).map_err(|e| e.into())?.0;
				let c = Compact::new(n, locations.as_slice());
				visitor.visit_compact_u16(c, outermost_ty_id)
			},
			TypeDef::Primitive(U32) => {
				locations.push(CompactLocation::Primitive(current_type_id));
				let n = codec::Compact::<u32>::decode(data).map_err(|e| e.into())?.0;
				let c = Compact::new(n, locations.as_slice());
				visitor.visit_compact_u32(c, outermost_ty_id)
			},
			TypeDef::Primitive(U64) => {
				locations.push(CompactLocation::Primitive(current_type_id));
				let n = codec::Compact::<u64>::decode(data).map_err(|e| e.into())?.0;
				let c = Compact::new(n, locations.as_slice());
				visitor.visit_compact_u64(c, outermost_ty_id)
			},
			TypeDef::Primitive(U128) => {
				locations.push(CompactLocation::Primitive(current_type_id));
				let n = codec::Compact::<u128>::decode(data).map_err(|e| e.into())?.0;
				let c = Compact::new(n, locations.as_slice());
				visitor.visit_compact_u128(c, outermost_ty_id)
			},
			// A struct with exactly 1 field containing one of the above types can be sensibly compact encoded/decoded.
			TypeDef::Composite(composite) => {
				if composite.fields.len() != 1 {
					return Err(DecodeError::CannotDecodeCompactIntoType(inner.clone()).into())
				}

				// What type is the 1 field that we are able to decode?
				let field = &composite.fields[0];

				// Record this composite location.
				match &field.name {
					Some(name) =>
						locations.push(CompactLocation::NamedComposite(current_type_id, name)),
					None => locations.push(CompactLocation::UnnamedComposite(current_type_id)),
				}

				let field_type_id = field.ty.id;
				let inner_ty = types
					.resolve(field_type_id)
					.ok_or(DecodeError::TypeIdNotFound(field_type_id))?;

				// Decode this inner type via compact decoding. This can recurse, in case
				// the inner type is also a 1-field composite type.
				decode_compact(
					data,
					outermost_ty_id,
					TypeId(field_type_id),
					locations,
					inner_ty,
					types,
					visitor,
				)
			},
			// For now, we give up if we have been asked for any other type:
			_cannot_decode_from =>
				Err(DecodeError::CannotDecodeCompactIntoType(inner.clone()).into()),
		}
	}

	// The type ID of the thing encoded into a Compact type.
	let inner_ty_id = ty.type_param.id;

	// Attempt to compact-decode this inner type.
	let inner = types.resolve(inner_ty_id).ok_or(DecodeError::TypeIdNotFound(inner_ty_id))?;

	// Track any inner type IDs we encounter.
	let locations = smallvec::SmallVec::<[CompactLocation; 8]>::new();

	decode_compact(data, ty_id, TypeId(inner_ty_id), locations, inner, types, visitor)
}

fn decode_bit_sequence_value<'scale, 'info, V: Visitor>(
	data: &mut &'scale [u8],
	ty_id: TypeId,
	ty: &'info TypeDefBitSequence<PortableForm>,
	types: &'info PortableRegistry,
	visitor: V,
) -> Result<V::Value<'scale, 'info>, V::Error> {
	use scale_bits::Format;

	let format = Format::from_metadata(ty, types).map_err(DecodeError::BitSequenceError)?;
	let mut bitseq = BitSequence::new(format, data);
	let res = visitor.visit_bitsequence(&mut bitseq, ty_id);

	// Move to the bytes after the bit sequence.
	*data = bitseq.bytes_after()?;

	res
}
