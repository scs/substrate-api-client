/*
   Copyright 2019 Supercomputing Systems AG

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

//! Primitives for substrate extrinsics.

use crate::OpaqueExtrinsic;
use alloc::{format, vec::Vec};
use codec::{Decode, Encode, Error, Input};
use core::fmt;
use scale_info::TypeInfo;
#[allow(deprecated)]
use sp_runtime::traits::Extrinsic;

/// Current version of the [`UncheckedExtrinsic`] encoded format.
const V4: u8 = 4;

#[allow(deprecated)]
pub mod deprecated {
	use super::*;
	/// Mirrors the currently used Extrinsic format (V4) from substrate. Has less traits and methods though.
	/// The SignedExtra used does not need to implement SignedExtension here.
	// see https://github.com/paritytech/substrate/blob/7d233c2446b5a60662400a0a4bcfb78bb3b79ff7/primitives/runtime/src/generic/unchecked_extrinsic.rs
	#[deprecated = "Use the `UncheckedExtrinsic` with Version 5 instead"]
	#[derive(Clone, Eq, PartialEq)]

	pub struct UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra> {
		/// The signature, address, number of extrinsics have come before from
		/// the same signer and an era describing the longevity of this transaction,
		/// if this is a signed extrinsic.
		pub signature: Option<(Address, Signature, SignedExtra)>,
		/// The function that should be called.
		pub function: Call,
	}

	impl<Address, Call, Signature, SignedExtra>
		UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>
	{
		/// New instance of a signed extrinsic.
		pub fn new_signed(
			function: Call,
			signed: Address,
			signature: Signature,
			extra: SignedExtra,
		) -> Self {
			UncheckedExtrinsicV4 { signature: Some((signed, signature, extra)), function }
		}

		/// New instance of an unsigned extrinsic.
		pub fn new_unsigned(function: Call) -> Self {
			Self { signature: None, function }
		}
	}

	impl<Address, Call, Signature, SignedExtra> Extrinsic
		for UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>
	where
		Address: TypeInfo,
		Signature: TypeInfo,
		Call: TypeInfo,
		SignedExtra: TypeInfo,
	{
		type Call = Call;

		type SignaturePayload = (Address, Signature, SignedExtra);

		fn is_signed(&self) -> Option<bool> {
			Some(self.signature.is_some())
		}

		fn new(function: Call, signed_data: Option<Self::SignaturePayload>) -> Option<Self> {
			Some(if let Some((address, signature, extra)) = signed_data {
				Self::new_signed(function, address, signature, extra)
			} else {
				Self::new_unsigned(function)
			})
		}
	}

	impl<Address, Call, Signature, SignedExtra> fmt::Debug
		for UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>
	where
		Address: fmt::Debug,
		Signature: fmt::Debug,
		Call: fmt::Debug,
		SignedExtra: fmt::Debug,
	{
		fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
			write!(
				f,
				"UncheckedExtrinsic({:?}, {:?})",
				self.signature.as_ref().map(|x| (&x.0, &x.2)),
				self.function
			)
		}
	}

	// https://github.com/paritytech/substrate/blob/1612e39131e3fe57ba4c78447fb1cbf7c4f8830e/primitives/runtime/src/generic/unchecked_extrinsic.rs#L289C5-L320
	impl<Address, Call, Signature, SignedExtra> Encode
		for UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>
	where
		Address: Encode,
		Signature: Encode,
		Call: Encode,
		SignedExtra: Encode,
	{
		fn encode(&self) -> Vec<u8> {
			encode_with_vec_prefix::<Self, _>(|v| {
				match self.signature.as_ref() {
					Some(s) => {
						v.push(V4 | 0b1000_0000);
						s.encode_to(v);
					},
					None => {
						v.push(V4 & 0b0111_1111);
					},
				}
				self.function.encode_to(v);
			})
		}
	}

	// https://github.com/paritytech/substrate/blob/1612e39131e3fe57ba4c78447fb1cbf7c4f8830e/primitives/runtime/src/generic/unchecked_extrinsic.rs#L250-L287
	impl<Address, Call, Signature, SignedExtra> Decode
		for UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>
	where
		Address: Decode,
		Signature: Decode,
		Call: Decode,
		SignedExtra: Decode,
	{
		fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
			// This is a little more complicated than usual since the binary format must be compatible
			// with substrate's generic `Vec<u8>` type. Basically this just means accepting that there
			// will be a prefix of vector length (we don't need
			// to use this).
			let _length_do_not_remove_me_see_above: Vec<()> = Decode::decode(input)?;

			let version = input.read_byte()?;

			let is_signed = version & 0b1000_0000 != 0;
			let version = version & 0b0111_1111;
			if version != V4 {
				return Err("Invalid transaction version".into())
			}

			Ok(UncheckedExtrinsicV4 {
				signature: if is_signed { Some(Decode::decode(input)?) } else { None },
				function: Decode::decode(input)?,
			})
		}
	}

	impl<Address, Call, Signature, SignedExtra> serde::Serialize
		for UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>
	where
		Address: Encode,
		Signature: Encode,
		Call: Encode,
		SignedExtra: Encode,
	{
		fn serialize<S>(&self, seq: S) -> Result<S::Ok, S::Error>
		where
			S: ::serde::Serializer,
		{
			self.using_encoded(|bytes| impl_serde::serialize::serialize(bytes, seq))
		}
	}

	// https://github.com/paritytech/substrate/blob/1612e39131e3fe57ba4c78447fb1cbf7c4f8830e/primitives/runtime/src/generic/unchecked_extrinsic.rs#L346-L357
	impl<'a, Address, Call, Signature, SignedExtra> serde::Deserialize<'a>
		for UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>
	where
		Address: Decode,
		Signature: Decode,
		Call: Decode,
		SignedExtra: Decode,
	{
		fn deserialize<D>(de: D) -> Result<Self, D::Error>
		where
			D: serde::Deserializer<'a>,
		{
			let r = impl_serde::serialize::deserialize(de)?;
			Decode::decode(&mut &r[..])
				.map_err(|e| serde::de::Error::custom(format!("Decode error: {e}")))
		}
	}

	// https://github.com/paritytech/substrate/blob/1612e39131e3fe57ba4c78447fb1cbf7c4f8830e/primitives/runtime/src/generic/unchecked_extrinsic.rs#L376-L390
	impl<Address, Call, Signature, Extra>
		From<UncheckedExtrinsicV4<Address, Call, Signature, Extra>> for OpaqueExtrinsic
	where
		Address: Encode,
		Signature: Encode,
		Call: Encode,
		Extra: Encode,
	{
		fn from(extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, Extra>) -> Self {
			Self::from_bytes(extrinsic.encode().as_slice()).expect(
			"both OpaqueExtrinsic and UncheckedExtrinsic have encoding that is compatible with \
				raw Vec<u8> encoding; qed",
		)
		}
	}

	/// Same function as in primitives::generic. Needed to be copied as it is private there.
	fn encode_with_vec_prefix<T: Encode, F: Fn(&mut Vec<u8>)>(encoder: F) -> Vec<u8> {
		let size = core::mem::size_of::<T>();
		let reserve = match size {
			0..=0b0011_1111 => 1,
			0b0100_0000..=0b0011_1111_1111_1111 => 2,
			_ => 4,
		};
		let mut v = Vec::with_capacity(reserve + size);
		v.resize(reserve, 0);
		encoder(&mut v);

		// need to prefix with the total length to ensure it's binary compatible with
		// Vec<u8>.
		let mut length: Vec<()> = Vec::new();
		length.resize(v.len() - reserve, ());
		length.using_encoded(|s| {
			v.splice(0..reserve, s.iter().cloned());
		});

		v
	}
}
