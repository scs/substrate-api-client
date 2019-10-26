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

use serde_json::{json, Value};

pub const REQUEST_TRANSFER: u32 = 3;

pub fn chain_get_block_hash() -> Value {
    chain_get_block_hash_with_id(1)
}

pub fn chain_get_block_hash_with_id(id: u32) -> Value {
    json!({
    "method": "chain_getBlockHash",
    "params": [0],
    "jsonrpc": "2.0",
    "id": id.to_string(),
    })
}

pub fn state_get_metadata() -> Value {
    state_get_metadata_with_id(1)
}

pub fn state_get_metadata_with_id(id: u32) -> Value {
    json!({
        "method": "state_getMetadata",
        "params": null,
        "jsonrpc": "2.0",
        "id": id.to_string(),
    })
}

pub fn state_get_runtime_version() -> Value {
    state_get_runtime_version_with_id(1)
}

pub fn state_get_runtime_version_with_id(id: u32) -> Value {
    json!({
        "method": "state_getRuntimeVersion",
        "params": null,
        "jsonrpc": "2.0",
        "id": id.to_string(),
    })
}

pub fn state_subscribe_storage(key: &str) -> Value {
    state_subscribe_storage_with_id(key, 1)
}

pub fn state_subscribe_storage_with_id(key: &str, id: u32) -> Value {
    json!({
        "method": "state_subscribeStorage",
        "params": [[key]],
        "jsonrpc": "2.0",
        "id": id.to_string(),
    })
}

pub fn state_get_storage(key_hash: &str) -> Value {
    state_get_storage_with_id(key_hash, 1)
}

pub fn state_get_storage_with_id(key_hash: &str, id: u32) -> Value {
    json_req("state_getStorage", key_hash, id)
}

pub fn author_submit_and_watch_extrinsic(xthex_prefixed: &str) -> Value {
    author_submit_and_watch_extrinsic_with_id(xthex_prefixed, REQUEST_TRANSFER)
}

pub fn author_submit_and_watch_extrinsic_with_id(xthex_prefixed: &str, id: u32) -> Value {
    json_req(
        "author_submitAndWatchExtrinsic",
        xthex_prefixed,
        id,
    )
}

fn json_req(method: &str, params: &str, id: u32) -> Value {
    json!({
        "method": method,
        "params": [params],
        "jsonrpc": "2.0",
        "id": id.to_string(),
    })
}
