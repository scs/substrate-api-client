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

use rstd::prelude::*;
use codec::Compact;
use primitives::H256 as Hash;


#[cfg(feature = "std")]
use crate::{Api,compose_extrinsic};

use super::xt_primitives::*;

pub const CONTRACTS_MODULE: &str = "Contract";
pub const CONTRACTS_PUT_CODE: &str = "put_code";
pub const CONTRACTS_CREATE: &str = "create";
pub const CONTRACTS_CALL: &str = "call";

pub type ContractPutCodeFn = ([u8; 2], Compact<u64>, Vec<u8>);
pub type ContractCreateFn = ([u8; 2], Compact<u128>, Compact<u64>, Hash, Vec<u8>);
pub type ContractCallFn = (
    [u8; 2],
    GenericAddress,
    Compact<u128>,
    Compact<u64>,
    Vec<u8>,
);

pub type ContractPutCodeXt = UncheckedExtrinsicV3<ContractPutCodeFn>;
pub type ContractCreateXt = UncheckedExtrinsicV3<ContractCreateFn>;
pub type ContractCallXt = UncheckedExtrinsicV3<ContractCallFn>;

#[cfg(feature = "std")]
pub fn put_code(api: Api, gas_limit: u64, code: Vec<u8>) -> ContractPutCodeXt {
    compose_extrinsic!(
        api,
        CONTRACTS_MODULE,
        CONTRACTS_PUT_CODE,
        Compact(gas_limit),
        code
    )
}


#[cfg(feature = "std")]
pub fn create(
    api: Api,
    endowment: u128,
    gas_limit: u64,
    code_hash: Hash,
    data: Vec<u8>,
) -> ContractCreateXt {

    compose_extrinsic!(
        api,
        CONTRACTS_MODULE,
        CONTRACTS_CREATE,
        Compact(endowment),
        Compact(gas_limit),
        code_hash,
        data
    )
}


#[cfg(feature = "std")]
pub fn call(
    api: Api,
    dest: GenericAddress,
    value: u128,
    gas_limit: u64,
    data: Vec<u8>,
) -> ContractCallXt {

    compose_extrinsic!(
        api,
        CONTRACTS_MODULE,
        CONTRACTS_CALL,
        dest,
        Compact(value),
        Compact(gas_limit),
        data
    )
}
