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

use codec::Compact;
use sp_core::crypto::Pair;
use sp_core::H256 as Hash;
use sp_std::prelude::*;
use sp_runtime::MultiSignature;

#[cfg(feature = "std")]
use crate::{compose_extrinsic, Api};

use super::xt_primitives::*;

pub const CONTRACTS_MODULE: &str = "Contract";
pub const CONTRACTS_PUT_CODE: &str = "put_code";
pub const CONTRACTS_INSTANTIATE: &str = "instantiate";
pub const CONTRACTS_CALL: &str = "call";

pub type ContractPutCodeFn = ([u8; 2], Compact<u64>, Vec<u8>);
pub type ContractInstantiateFn = ([u8; 2], Compact<u128>, Compact<u64>, Hash, Vec<u8>);
pub type ContractCallFn = (
    [u8; 2],
    GenericAddress,
    Compact<u128>,
    Compact<u64>,
    Vec<u8>,
);

pub type ContractPutCodeXt = UncheckedExtrinsicV4<ContractPutCodeFn>;
pub type ContractInstantiateXt = UncheckedExtrinsicV4<ContractInstantiateFn>;
pub type ContractCallXt = UncheckedExtrinsicV4<ContractCallFn>;

#[cfg(feature = "std")]
impl<P> Api<P>
where
    P: Pair,
    MultiSignature: From<P::Signature>,
{
    pub fn contract_put_code(&self, gas_limit: u64, code: Vec<u8>) -> ContractPutCodeXt {
        compose_extrinsic!(
            &self,
            CONTRACTS_MODULE,
            CONTRACTS_PUT_CODE,
            Compact(gas_limit),
            code
        )
    }

    pub fn contract_instantiate(
        &self,
        endowment: u128,
        gas_limit: u64,
        code_hash: Hash,
        data: Vec<u8>,
    ) -> ContractInstantiateXt {

            compose_extrinsic!(
            self,
            CONTRACTS_MODULE,
            CONTRACTS_INSTANTIATE,
            Compact(endowment),
            Compact(gas_limit),
            code_hash,
            data
        )
    }

    pub fn contract_call(
        &self,
        dest: GenericAddress,
        value: u128,
        gas_limit: u64,
        data: Vec<u8>,
    ) -> ContractCallXt {
        compose_extrinsic!(
            self,
            CONTRACTS_MODULE,
            CONTRACTS_CALL,
            dest,
            Compact(value),
            Compact(gas_limit),
            data
        )
    }
}
