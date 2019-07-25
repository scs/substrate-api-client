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



pub fn state_get_storage(key_hash: &str) -> Value {
    json!({
            "method": "state_getStorage",
            "params": [key_hash.to_string()],
            "jsonrpc": "2.0",
            "id": "1",
        })
}

pub fn author_submit_and_watch_extrinsic(xthex_prefixed: &str) -> Value {
    json!({
            "method": "author_submitAndWatchExtrinsic",
            "params": [xthex_prefixed],
            "jsonrpc": "2.0",
            "id": REQUEST_TRANSFER.to_string(),
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
