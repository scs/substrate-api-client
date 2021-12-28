use crate::std::rpc::XtStatus;
pub use ac_node_api::metadata::{InvalidMetadataError, MetadataError};

pub type ApiResult<T> = Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Fetching genesis hash failed. Are you connected to the correct endpoint?")]
    Genesis,
    #[error("Fetching runtime version failed. Are you connected to the correct endpoint?")]
    RuntimeVersion,
    #[error("Fetching Metadata failed. Are you connected to the correct endpoint?")]
    MetadataFetch,
    #[error("Operation needs a signer to be set in the api")]
    NoSigner,
    #[cfg(feature = "ws-client")]
    #[error("WebSocket Error: {0}")]
    WebSocket(#[from] ws::Error),
    #[error("RpcClient error: {0}")]
    RpcClient(String),
    #[error("ChannelReceiveError, sender is disconnected: {0}")]
    Disconnected(#[from] sp_std::sync::mpsc::RecvError),
    #[error("Metadata Error: {0}")]
    Metadata(#[from] MetadataError),
    #[error("InvalidMetadata: {0}")]
    InvalidMetadata(#[from] InvalidMetadataError),
    #[cfg(feature = "ws-client")]
    #[error("Events Error: {0}")]
    NodeApi(#[from] ac_node_api::error::Error),
    #[error("Error decoding storage value: {0}")]
    StorageValueDecode(#[from] codec::Error),
    #[error("Received invalid hex string: {0}")]
    InvalidHexString(#[from] hex::FromHexError),
    #[error("Error deserializing with serde: {0}")]
    Deserializing(#[from] serde_json::Error),
    #[error("UnsupportedXtStatus Error: Can only wait for finalized, in block, broadcast and ready. Waited for: {0:?}")]
    UnsupportedXtStatus(XtStatus),
    #[error("Error converting NumberOrHex to Balance")]
    TryFromIntError,
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}
