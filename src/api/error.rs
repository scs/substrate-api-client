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

use crate::{api::UnexpectedTxStatus, rpc::Error as RpcClientError};
use ac_node_api::{
	error::DispatchError,
	metadata::{InvalidMetadataError, MetadataError},
};
use alloc::boxed::Box;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, derive_more::From)]
pub enum Error {
	/// Could not fetch the genesis hash from node.
	FetchGenesisHash,
	/// Expected a signer, but none is assigned.
	NoSigner,
	/// Rpc Client Error.
	RpcClient(RpcClientError),
	/// Metadata Error.
	Metadata(MetadataError),
	/// Invalid Metadata Error.
	InvalidMetadata(InvalidMetadataError),
	/// Node Api Error.
	NodeApi(ac_node_api::error::Error),
	/// Encode / Decode Error.
	Codec(codec::Error),
	/// Could not convert NumberOrHex with try_from.
	TryFromIntError,
	/// Node Api Dispatch Error.
	Dispatch(DispatchError),
	/// Encountered unexpected tx status during watch process.
	UnexpectedTxStatus(UnexpectedTxStatus),
	/// Could not send update because the Stream has been closed unexpectedly.
	NoStream,
	/// Could not find the expected extrinsic.
	ExtrinsicNotFound,
	/// Could not find the expected block hash.
	BlockHashNotFound,
	/// Could not find the expected block.
	BlockNotFound,
	/// Any custom Error.
	Other(Box<dyn core::error::Error + Send + Sync + 'static>),
}
