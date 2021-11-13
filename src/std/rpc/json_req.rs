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

use serde::Serialize;
use serde_json::{json, to_value, Value};
use sp_core::storage::StorageKey;
use sp_core::H256 as Hash;

pub const REQUEST_TRANSFER: u32 = 3;

pub fn chain_get_header(hash: Option<Hash>) -> Value {
    json_req("chain_getHeader", vec![hash], 1)
}

pub fn chain_get_block_hash(number: Option<u32>) -> Value {
    chain_get_block_hash_with_id(number, 1)
}

pub fn chain_get_genesis_hash() -> Value {
    chain_get_block_hash(Some(0))
}

pub fn chain_get_block_hash_with_id(number: Option<u32>, id: u32) -> Value {
    json_req("chain_getBlockHash", vec![number], id)
}

pub fn chain_get_block(hash: Option<Hash>) -> Value {
    json_req("chain_getBlock", vec![hash], 1)
}

pub fn chain_get_finalized_head() -> Value {
    json_req("chain_getFinalizedHead", Value::Null, 1)
}

pub fn chain_subscribe_finalized_heads() -> Value {
    json_req("chain_subscribeFinalizedHeads", Value::Null, 1)
}

pub fn payment_query_fee_details(xthex_prefixed: &str, at_block: Option<Hash>) -> Value {
    json_req(
        "payment_queryFeeDetails",
        vec![
            to_value(xthex_prefixed).unwrap(),
            to_value(at_block).unwrap(),
        ],
        1,
    )
}

pub fn payment_query_info(xthex_prefixed: &str, at_block: Option<Hash>) -> Value {
    json_req(
        "payment_queryInfo",
        vec![
            to_value(xthex_prefixed).unwrap(),
            to_value(at_block).unwrap(),
        ],
        1,
    )
}

pub fn state_get_metadata() -> Value {
    state_get_metadata_with_id(1)
}

pub fn state_get_metadata_with_id(id: u32) -> Value {
    json_req("state_getMetadata", vec![Value::Null], id)
}

pub fn state_get_runtime_version() -> Value {
    state_get_runtime_version_with_id(1)
}

pub fn state_get_runtime_version_with_id(id: u32) -> Value {
    json_req("state_getRuntimeVersion", vec![Value::Null], id)
}

pub fn state_subscribe_storage(key: Vec<StorageKey>) -> Value {
    state_subscribe_storage_with_id(key, 1)
}

pub fn state_subscribe_storage_with_id(key: Vec<StorageKey>, id: u32) -> Value {
    // don't know why we need an additional vec here...
    json_req("state_subscribeStorage", vec![key], id)
}

pub fn state_get_storage(key: StorageKey, at_block: Option<Hash>) -> Value {
    json_req(
        "state_getStorage",
        vec![to_value(key).unwrap(), to_value(at_block).unwrap()],
        1,
    )
}

pub fn state_get_storage_with_id(key: StorageKey, at_block: Option<Hash>, id: u32) -> Value {
    json_req(
        "state_getStorage",
        vec![to_value(key).unwrap(), to_value(at_block).unwrap()],
        id,
    )
}

pub fn state_get_read_proof(keys: Vec<StorageKey>, at_block: Option<Hash>) -> Value {
    json_req(
        "state_getReadProof",
        vec![to_value(keys).unwrap(), to_value(at_block).unwrap()],
        1,
    )
}

pub fn state_get_keys(key: StorageKey, at_block: Option<Hash>) -> Value {
    json_req(
        "state_getKeys",
        vec![to_value(key).unwrap(), to_value(at_block).unwrap()],
        1,
    )
}

pub fn author_submit_extrinsic(xthex_prefixed: &str) -> Value {
    author_submit_extrinsic_with_id(xthex_prefixed, REQUEST_TRANSFER)
}

pub fn author_submit_and_watch_extrinsic(xthex_prefixed: &str) -> Value {
    author_submit_and_watch_extrinsic_with_id(xthex_prefixed, REQUEST_TRANSFER)
}

pub fn author_submit_extrinsic_with_id(xthex_prefixed: &str, id: u32) -> Value {
    json_req("author_submitExtrinsic", vec![xthex_prefixed], id)
}

pub fn author_submit_and_watch_extrinsic_with_id(xthex_prefixed: &str, id: u32) -> Value {
    json_req("author_submitAndWatchExtrinsic", vec![xthex_prefixed], id)
}

fn json_req<S: Serialize>(method: &str, params: S, id: u32) -> Value {
    json!({
        "method": method,
        "params": params,
        "jsonrpc": "2.0",
        "id": id.to_string(),
    })
}
