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

use std::fmt;

use codec::{Compact, Decode, Encode};
use indices::address::Address;
use node_primitives::Signature;
use runtime_primitives::generic::Era;

pub type GenericAddress = Address<[u8; 32], u32>;

/// Simple generic extra mirroring the SignedExtra currently used in extrinsics. Does not implement
/// the SignedExtension trait. It simply encodes to the same bytes as the real SignedExtra.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct GenericExtra(Era, Compact<u32>, Compact<u128>);

impl GenericExtra {
    pub fn new(nonce: u32) -> GenericExtra {
        GenericExtra(
            Era::Immortal,
            Compact(nonce),
            Compact(0 as u128),
        )
    }
}

/// Mirrors the currently used Extrinsic format (V3) from substrate. Has less traits and methods though.
/// The SingedExtra used does not need to implement SingedExtension here.
pub struct UncheckedExtrinsicV3<Call>
    where
        Call: Encode + fmt::Debug,
{
    pub signature: Option<(GenericAddress, Signature, GenericExtra)>,
    pub function: Call,
}

impl<Call> UncheckedExtrinsicV3<Call>
    where
        Call: Encode + fmt::Debug,
{
    pub fn new_signed(
        function: Call,
        signed: GenericAddress,
        signature: Signature,
        extra: GenericExtra,
    ) -> Self {
        UncheckedExtrinsicV3 {
            signature: Some((signed, signature, extra)),
            function,
        }
    }

    pub fn hex_encode(&self) -> String {
        let mut hex_str = hex::encode(self.encode());
        hex_str.insert_str(0, "0x");
        hex_str
    }
}

impl<Call> fmt::Debug for UncheckedExtrinsicV3<Call>
    where
        Call: fmt::Debug + Encode,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UncheckedExtrinsic({:?}, {:?})", self.signature.as_ref().map(|x| (&x.0, &x.2)), self.function)
    }
}

impl<Call> Encode for UncheckedExtrinsicV3<Call>
    where
        Call: Encode + fmt::Debug,
{
    fn encode(&self) -> Vec<u8> {
        encode_with_vec_prefix::<Self, _>(|v| {
            match self.signature.as_ref() {
                Some(s) => {
                    v.push(3 as u8 | 0b1000_0000);
                    s.encode_to(v);
                },
                None => {
                    v.push(3 as u8 & 0b0111_1111);
                }
            }
            self.function.encode_to(v);
        })
    }
}

/// Same function as in primitives::generic. Needed to be copied as it is private there.
fn encode_with_vec_prefix<T: Encode, F: Fn(&mut Vec<u8>)>(encoder: F) -> Vec<u8> {
    let size = std::mem::size_of::<T>();
    let reserve = match size {
        0..=0b0011_1111 => 1,
        0..=0b0011_1111_1111_1111 => 2,
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