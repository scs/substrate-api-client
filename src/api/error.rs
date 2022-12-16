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

use crate::{api::XtStatus, rpc::Error as RpcClientError};
use ac_node_api::{
	metadata::{InvalidMetadataError, MetadataError},
	DispatchError,
};

pub type Result<T> = core::result::Result<T, Error>;

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
	#[error("RpcClient error: {0:?}")]
	RpcClient(#[from] RpcClientError),
	#[error("ChannelReceiveError, sender is disconnected: {0}")]
	Disconnected(#[from] sp_std::sync::mpsc::RecvError),
	#[error("Metadata Error: {0:?}")]
	Metadata(MetadataError),
	#[error("InvalidMetadata: {0:?}")]
	InvalidMetadata(InvalidMetadataError),
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
	#[error("Extrinsic Error: {0}")]
	Extrinsic(String),
	#[error("Stream ended unexpectedly")]
	NoStream,
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

impl From<ac_node_api::error::Error> for Error {
	fn from(error: ac_node_api::error::Error) -> Self {
		Error::NodeApi(error)
	}
}
