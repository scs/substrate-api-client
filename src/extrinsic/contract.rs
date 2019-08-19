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
use node_primitives::Hash;
use primitive_types::U256;

use crate::crypto::AccountKey;
use crate::node_metadata::NodeMetadata;
use crate::compose_extrinsic;

use super::xt_primitives::*;

pub const CONTRACTS_MODULE: &str = "Contract";
pub const CONTRACTS_PUT_CODE: &str = "put_code";
pub const CONTRACTS_CREATE: &str = "create";
pub const CONTRACTS_CALL: &str = "call";

pub type ContractPutCodeFn = ([u8; 2], Compact<u64>, Vec<u8>);
pub type ContractCreateFn = ([u8; 2], Compact<u128>, Compact<u64>, Hash, Vec<u8>);
pub type ContractCallFn = ([u8; 2], GenericAddress, Compact<u128>, Compact<u64>, Vec<u8>);

pub type ContractPutCodeXt = UncheckedExtrinsicV3<ContractPutCodeFn>;
pub type ContractCreateXt = UncheckedExtrinsicV3<ContractCreateFn>;
pub type ContractCallXt = UncheckedExtrinsicV3<ContractCallFn>;

pub fn put_code(from: AccountKey, gas_limit: u64, code: Vec<u8>, nonce: U256, genesis_hash: Hash, node_metadata: NodeMetadata) -> ContractPutCodeXt {
    compose_extrinsic!(
		node_metadata,
		genesis_hash,
		CONTRACTS_MODULE,
		CONTRACTS_PUT_CODE,
		GenericExtra::new(nonce.low_u32()),
		from,
		Compact(gas_limit),
		code
	)
}

pub fn create(from: AccountKey, endowment: u128, gas_limit: u64, code_hash: Hash, data: Vec<u8>, nonce: U256, genesis_hash: Hash, node_metadata: NodeMetadata) -> ContractCreateXt {
    compose_extrinsic!(
		node_metadata,
		genesis_hash,
		CONTRACTS_MODULE,
		CONTRACTS_CREATE,
		GenericExtra::new(nonce.low_u32()),
		from,
		Compact(endowment),
		Compact(gas_limit),
		code_hash,
		data
	)
}

pub fn call(from: AccountKey, dest: GenericAddress, value: u128, gas_limit: u64, data: Vec<u8>, nonce: U256, genesis_hash: Hash, node_metadata: NodeMetadata) -> ContractCallXt {
    compose_extrinsic!(
		node_metadata,
		genesis_hash,
		CONTRACTS_MODULE,
		CONTRACTS_CALL,
		GenericExtra::new(nonce.low_u32()),
		from,
		dest,
		Compact(value),
		Compact(gas_limit),
		data
	)
}