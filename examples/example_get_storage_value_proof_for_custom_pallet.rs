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

use clap::{load_yaml, App};

use sp_core::{sr25519, twox_128};
use substrate_api_client::rpc::{ReadProof, WsRpcClient};
use substrate_api_client::Api;
use substrate_api_client::PlainTipExtrinsicParams;

use ac_node_api::Metadata;
use codec::{Decode, Encode};
use jsonrpsee::{
    client_transport::ws::{Uri, WsTransportClientBuilder},
    core::client::{ClientBuilder, ClientT},
};
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Value};
use sp_core::RuntimeDebug;

///For this example to run, run a polkadex-node locally, with:
/// docker run -p 9944:9944 -p 9933:9933 -p 30333:30333 polkadex/mainnet:v4.0.0  --dev --ws-external --rpc-external
#[tokio::main]
async fn main() {
    env_logger::init();

    // get storage proof from api client
    let url = get_node_url_from_cli();

    let client = WsRpcClient::new(&url);
    let api = Api::<sr25519::Pair, _, PlainTipExtrinsicParams>::new(client).unwrap();

    let block_hash = api.get_block_hash(None).unwrap();

    let proof: Option<ReadProof<H256>> = api
        .get_storage_value_proof("OCEX", "IngressMessages", block_hash)
        .unwrap();
    println!("[+] StorageValueProof: {:?}", proof);

    // get storage proof from jsonrpseeclient
    let metadata = api.get_metadata().map(Metadata::try_from).unwrap().unwrap();
    let storage_key = metadata
        .storage_value_key("OCEX", "IngressMessages")
        .unwrap();

    let uri: Uri = "ws://127.0.0.1:9944".parse().unwrap();
    let (tx, rx) = WsTransportClientBuilder::default()
        .build(uri)
        .await
        .unwrap();
    let client2 = ClientBuilder::default().build_with_tokio(tx, rx);

    let ingress_messages: Value = client2
        .request(
            "state_getReadProof",
            jsonrpsee::rpc_params![
                vec![to_value(storage_key.clone()).unwrap()],
                block_hash.clone()
            ],
        )
        .await
        .unwrap();
    let proof_jsonrpsee: ReadProof<H256> =
        serde_json::from_str(&ingress_messages.to_string()).unwrap();
    println!(
        "[+] StorageValueProof from JsonRpsee: {:?}",
        proof_jsonrpsee
    );

    assert_eq!(proof.unwrap(), proof_jsonrpsee, "Proofs are not equal");
}

pub fn get_node_url_from_cli() -> String {
    let yml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yml).get_matches();

    let node_ip = matches.value_of("node-server").unwrap_or("ws://127.0.0.1");
    let node_port = matches.value_of("node-port").unwrap_or("9944");
    let url = format!("{}:{}", node_ip, node_port);
    println!("Interacting with node on {}\n", url);
    url
}

/// Storage key.
// https://github.com/paritytech/substrate/blob/cd2fdcf85eb96c53ce2a5d418d4338eb92f5d4f5/primitives/storage/src/lib.rs#L41-L43
#[derive(
    PartialEq,
    Eq,
    RuntimeDebug,
    Serialize,
    Deserialize,
    Hash,
    PartialOrd,
    Ord,
    Clone,
    Encode,
    Decode,
)]
pub struct StorageKey(#[serde(with = "impl_serde::serialize")] pub Vec<u8>);

/// Returns the concacenated 128 bit hash of the given module and specific storage key
/// as a full Substrate StorageKey.
pub fn storage_key(module: &str, storage_key_name: &str) -> StorageKey {
    let mut key = twox_128(module.as_bytes()).to_vec();
    key.extend(twox_128(storage_key_name.as_bytes()));
    StorageKey(key)
}

impl AsRef<[u8]> for StorageKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<StorageKey> for sp_core::storage::StorageKey {
    fn from(storage_key: StorageKey) -> Self {
        Self(storage_key.0)
    }
}

impl From<sp_core::storage::StorageKey> for StorageKey {
    fn from(storage_key: sp_core::storage::StorageKey) -> Self {
        Self(storage_key.0)
    }
}
