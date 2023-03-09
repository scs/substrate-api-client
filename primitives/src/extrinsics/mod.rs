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

use alloc::vec::Vec;
use codec::{Decode, Encode, Error, Input};
use core::fmt;

pub use extrinsic_params::{
	AssetTip, AssetTipExtrinsicParams, ExtrinsicParams, GenericAdditionalParams,
	GenericAdditionalSigned, GenericExtrinsicParams, GenericSignedExtra, PlainTip,
	PlainTipExtrinsicParams, SignedPayload,
};
pub use signer::{ExtrinsicSigner, SignExtrinsic};

/// Call Index used a prefix of every extrinsic call.
pub type CallIndex = [u8; 2];

pub mod extrinsic_params;
pub mod signer;

/// Current version of the [`UncheckedExtrinsic`] encoded format.
const V4: u8 = 4;

/// Mirrors the currently used Extrinsic format (V4) from substrate. Has less traits and methods though.
/// The SingedExtra used does not need to implement SingedExtension here.
// see https://github.com/paritytech/substrate/blob/7d233c2446b5a60662400a0a4bcfb78bb3b79ff7/primitives/runtime/src/generic/unchecked_extrinsic.rs
#[derive(Clone, Eq, PartialEq)]
pub struct UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra> {
	pub signature: Option<(Address, Signature, SignedExtra)>,
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

#[cfg(test)]
mod tests {
	use super::*;
	use extrinsic_params::{
		ExtrinsicParams, GenericAdditionalParams, GenericExtrinsicParams, PlainTip,
	};
	use sp_core::{Pair, H256 as Hash};
	use sp_runtime::{generic::Era, testing::sr25519, AccountId32, MultiSignature};

	#[test]
	fn encode_decode_roundtrip_works() {
		let msg = &b"test-message"[..];
		let (pair, _) = sr25519::Pair::generate();
		let signature = pair.sign(msg);
		let multi_sig = MultiSignature::from(signature);
		let account: AccountId32 = pair.public().into();
		let tx_params = GenericAdditionalParams::<PlainTip<u128>, Hash>::new()
			.era(Era::mortal(8, 0), Hash::from([0u8; 32]));

		let default_extra =
			GenericExtrinsicParams::new(0, 0, 0u32, Hash::from([0u8; 32]), tx_params);
		let xt = UncheckedExtrinsicV4::new_signed(
			vec![1, 1, 1],
			account,
			multi_sig,
			default_extra.signed_extra(),
		);
		let xt_enc = xt.encode();
		assert_eq!(xt, Decode::decode(&mut xt_enc.as_slice()).unwrap())
	}
}
