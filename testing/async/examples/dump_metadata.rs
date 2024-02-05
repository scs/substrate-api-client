use sp_core::Bytes;
use std::{fs::File, io::prelude::*};
use substrate_api_client::{
	ac_compose_macros::rpc_params,
	rpc::{JsonrpseeClient, Request},
};

// Simple file to dump new metdata as a file.
// Run with: cargo run -p ac-testing --example dump_metadata

#[tokio::main]
async fn main() {
	let client = JsonrpseeClient::new("wss://kusama-rpc.polkadot.io:443").await.unwrap();
	let metadata_bytes: Bytes = client.request("state_getMetadata", rpc_params![]).await.unwrap();
	let mut file = File::create("new_ksm_metadata.bin").unwrap();
	file.write_all(&metadata_bytes.0).unwrap();
}
