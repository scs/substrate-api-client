
use substrate_api_client::{
	rpc::JsonrpseeClient, rpc::Request, ac_compose_macros::rpc_params,
};
use std::fs::File;
use sp_core::Bytes;
use std::io::prelude::*;

// Simple file to dump new metdata as a file.
// Run with: cargo un -p ac-testing --example dump_metadata

#[tokio::main]
async fn main() {
	let client = JsonrpseeClient::new("wss://kusama-rpc.polkadot.io:443").unwrap();
	let metadata_bytes: Bytes = client.request("state_getMetadata", rpc_params![]).unwrap();
	let mut file = File::create("new_ksm_metadata.bin").unwrap();
    file.write_all(&metadata_bytes.0).unwrap();
}
