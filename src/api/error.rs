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
use alloc::{boxed::Box, string::String};

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
	#[error("Metadata Error: {0:?}")]
	Metadata(MetadataError),
	#[error("InvalidMetadata: {0:?}")]
	InvalidMetadata(InvalidMetadataError),
	#[error("Events Error: {0:?}")]
	NodeApi(ac_node_api::error::Error),
	#[error("Error decoding storage value: {0}")]
	StorageValueDecode(codec::Error),
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
	#[error("Expected a block hash")]
	NoBlockHash,
	#[error("Did not find any block")]
	NoBlock,
	#[error(transparent)]
	Other(#[from] Box<dyn core::error::Error + Send + Sync + 'static>),
}

impl From<codec::Error> for Error {
	fn from(error: codec::Error) -> Self {
		Error::StorageValueDecode(error)
	}
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

impl From<ac_node_api::error::DispatchError> for Error {
	fn from(error: ac_node_api::error::DispatchError) -> Self {
		Error::Dispatch(error)
	}
}
