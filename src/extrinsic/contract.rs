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
use sp_runtime::{MultiSignature, MultiSigner};
use sp_std::prelude::*;

#[cfg(feature = "std")]
use crate::{
    compose_extrinsic,
    std::{Api, RpcClient},
};

use super::xt_primitives::*;

pub const CONTRACTS_MODULE: &str = "Contract";
pub const CONTRACTS_PUT_CODE: &str = "put_code";
pub const CONTRACTS_INSTANTIATE: &str = "instantiate";
pub const CONTRACTS_CALL: &str = "call";

type CallIndex = [u8; 2];

type Gas = u64;
type Data = Vec<u8>;
type Balance = u128;

type GasLimit = Compact<Gas>;
type Endowment = Compact<Balance>;
type Value = Compact<Balance>;
type Destination = GenericAddress;

pub type ContractPutCodeFn = (CallIndex, GasLimit, Data);
pub type ContractInstantiateFn = (CallIndex, Endowment, GasLimit, Hash, Data);
pub type ContractCallFn = (CallIndex, Destination, Value, GasLimit, Data);

pub type ContractPutCodeXt = UncheckedExtrinsicV4<ContractPutCodeFn>;
pub type ContractInstantiateXt = UncheckedExtrinsicV4<ContractInstantiateFn>;
pub type ContractCallXt = UncheckedExtrinsicV4<ContractCallFn>;

#[cfg(feature = "std")]
impl<P, Client> Api<P, Client>
where
    P: Pair,
    MultiSignature: From<P::Signature>,
    MultiSigner: From<P::Public>,
    Client: RpcClient,
{
    pub fn contract_put_code(&self, gas_limit: Gas, code: Data) -> ContractPutCodeXt {
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
        endowment: Balance,
        gas_limit: Gas,
        code_hash: Hash,
        data: Data,
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
        value: Balance,
        gas_limit: Gas,
        data: Data,
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
