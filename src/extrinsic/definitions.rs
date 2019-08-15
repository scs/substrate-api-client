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

use indices::address::Address;
use node_primitives::Signature;
use codec::{Compact, Encode, Decode};
use runtime_primitives::traits::SignedExtension;
use runtime_primitives::generic::Era;
use std::fmt;

pub const BALANCES_MODULE_NAME: &str = "Balances";
pub const BALANCES_TRANSFER: &str = "transfer";

pub type GenericAddress = Address<[u8; 32], u32>;
//pub type UncheckedExtrinsicV3<F, E> = UncheckedExtrinsic<GenericAddress, F, Signature, E>;

pub type BalanceTransfer = ([u8; 2], GenericAddress, Compact<u128>);

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct GenericExtra {
    pub era: Era,
    pub nonce: u64,
    pub tip: u128,
}
//
//#[derive(Debug, Encode, Decode, Clone, Eq, PartialEq, Ord, PartialOrd)]
//pub struct GenericExtra;
impl SignedExtension for GenericExtra {
    type AccountId = u64;
    type Call = ();
    type AdditionalSigned = ();
    type Pre = ();

    fn additional_signed(&self) -> std::result::Result<(), &'static str> { Ok(()) }
}

pub type BalanceExtrinsic = UncheckedExtrinsicV3<BalanceTransfer, GenericExtra>;

pub struct UncheckedExtrinsicV3<Call, Extra>
where
    Call: Encode + fmt::Debug,
    Extra: Encode + fmt::Debug,
{
    pub signature: Option<(GenericAddress, Signature, Extra)>,
    pub function: Call,
}

impl<Call, Extra> fmt::Debug for UncheckedExtrinsicV3<Call, Extra>
    where
        Call: fmt::Debug + Encode,
        Extra: fmt::Debug + Encode,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UncheckedExtrinsic({:?}, {:?})", self.signature.as_ref().map(|x| (&x.0, &x.2)), self.function)
    }
}

impl<Call, Extra> Encode for UncheckedExtrinsicV3<Call, Extra>
where
    Call: Encode + fmt::Debug,
    Extra: SignedExtension,
{
    fn encode(&self) -> Vec<u8> {
        encode_with_vec_prefix::<Self, _>( |v| {
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

fn encode_with_vec_prefix<T: Encode, F: Fn(&mut Vec<u8>)>(encoder: F) -> Vec<u8> {
    let size = std::mem::size_of::<T>();
    let reserve = match size {
        0..=0b00111111 => 1,
        0..=0b00111111_11111111 => 2,
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