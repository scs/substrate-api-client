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
use serde::{Deserialize, Serialize};

#[cfg(feature = "ws-client")]
pub use ws_client::WsRpcClient;

#[cfg(feature = "ws-client")]
pub mod ws_client;

pub mod json_req;

#[derive(Debug, thiserror::Error)]
pub enum RpcClientError {
    #[error("Serde json error: {0}")]
    Serde(#[from] serde_json::error::Error),
    #[error("Extrinsic Error: {0}")]
    Extrinsic(String),
    #[error("mpsc send Error: {0}")]
    Send(#[from] std::sync::mpsc::SendError<String>),
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum XtStatus {
    // Todo: some variants to not return a hash with `send_extrinsics`: #175.
    Finalized,
    InBlock,
    Broadcast,
    Ready,
    Future,
    /// uses `author_submit`
    SubmitOnly,
    Error,
    Unknown,
}

// Exact structure from
// https://github.com/paritytech/substrate/blob/master/client/rpc-api/src/state/helpers.rs
// Adding manually so we don't need sc-rpc-api, which brings in async dependencies
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadProof<Hash> {
    /// Block hash used to generate the proof
    pub at: Hash,
    /// A proof used to prove that storage entries are included in the storage trie
    pub proof: Vec<sp_core::Bytes>,
}
