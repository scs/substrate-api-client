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
	Send(String),
	#[error("Expected some error information, but nothing was found: {0}")]
	NoErrorInformationFound(String),
	#[error(transparent)]
	Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum XtStatus {
	Unknown = 0,
	/// uses `author_submit` without watching.
	SubmitOnly = 1,
	Ready = 2,
	Broadcast = 3,
	InBlock = 4,
	Finalized = 5,
	Future = 10,
}

// Exact structure from
// https://github.com/paritytech/substrate/blob/master/client/rpc-api/src/state/helpers.rs
// Adding manually so we don't need sc-rpc-api, which brings in async dependencies
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadProof<Hash> {
	/// Block hash used to generate the proof
	pub at: Hash,
	/// A proof used to prove that storage entries are included in the storage trie
	pub proof: Vec<sp_core::Bytes>,
}
