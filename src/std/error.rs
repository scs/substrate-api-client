use crate::{rpc::RpcClientError, std::rpc::XtStatus};
use ac_node_api::{
	metadata::{InvalidMetadataError, MetadataError},
	DispatchError,
};

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
	#[error("RpcClient error: {0:?}")]
	RpcClient(#[from] RpcClientError),
	#[error("ChannelReceiveError, sender is disconnected: {0}")]
	Disconnected(#[from] sp_std::sync::mpsc::RecvError),
	#[error("Metadata Error: {0:?}")]
	Metadata(MetadataError),
	#[error("InvalidMetadata: {0:?}")]
	InvalidMetadata(InvalidMetadataError),
	#[cfg(feature = "ws-client")]
	#[error("Events Error: {0:?}")]
	NodeApi(ac_node_api::error::Error),
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
	#[error("The node runtime could not dispatch an extrinsic")]
	Dispatch(DispatchError),
	#[error(transparent)]
	Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl From<InvalidMetadataError> for Error {
	fn from(error: InvalidMetadataError) -> Self {
		Error::InvalidMetadata(error)
	}
}

impl From<MetadataError> for Error {
	fn from(error: MetadataError) -> Self {
		Error::Metadata(error)
	}
}

#[cfg(feature = "ws-client")]
impl From<ac_node_api::error::Error> for Error {
	fn from(error: ac_node_api::error::Error) -> Self {
		Error::NodeApi(error)
	}
}
