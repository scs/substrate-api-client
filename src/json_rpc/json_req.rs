use serde_json::{json, Value};

pub const REQUEST_TRANSFER: u32         = 3;

pub fn chain_get_block_hash() -> Value {
    json!({
            "method": "chain_getBlockHash",
            "params": [0],
            "jsonrpc": "2.0",
            "id": "1",
            })
}

pub fn state_get_metadata() -> Value {
    json!({
            "method": "state_getMetadata",
            "params": null,
            "jsonrpc": "2.0",
            "id": "1",
        })
}

pub fn state_subscribe_storage(key: &str) -> Value {
    json!({
            "method": "state_subscribeStorage",
            "params": [[key]],
            "jsonrpc": "2.0",
            "id": "1",
        })
}

pub fn state_get_storage(key_hash: &str) -> Value {
    json_req("state_getStorage", key_hash, 1 as u32)
}

pub fn author_submit_and_watch_extrinsic(xthex_prefixed: &str) -> Value {
    json_req("author_submitAndWatchExtrinsic", xthex_prefixed, REQUEST_TRANSFER)
}

fn json_req(method: &str, params: &str, id: u32) -> Value {
    json!({
            "method": method,
            "params": [params],
            "jsonrpc": "2.0",
            "id": id.to_string(),
        })
}