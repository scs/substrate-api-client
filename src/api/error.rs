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

use crate::{api::UnexpectedTxStatus, rpc::Error as RpcClientError, ExtrinsicReport};
use ac_node_api::{
	error::DispatchError,
	metadata::{MetadataConversionError, MetadataError},
};
use alloc::boxed::Box;
use codec::{Decode, Encode};

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
	InvalidMetadata(MetadataConversionError),
	/// Node Api Error.
	NodeApi(ac_node_api::error::Error),
	/// Encode / Decode Error.
	Codec(codec::Error),
	/// Could not convert NumberOrHex with try_from.
	TryFromIntError,
	/// Encountered unexpected tx status during watch process.
	UnexpectedTxStatus(UnexpectedTxStatus),
	/// Could not find the expected extrinsic.
	ExtrinsicNotFound,
	/// Could not find the expected block hash.
	BlockHashNotFound,
	/// Could not find the expected block.
	BlockNotFound,
	/// Any custom Error.
	Other(Box<dyn core::error::Error + Send + Sync + 'static>),
}

pub type ExtrinsicResult<T, Hash> = core::result::Result<T, ExtrinsicError<Hash>>;

/// Error Type returned upon submission or watch error.
#[derive(Debug, derive_more::From)]
pub enum ExtrinsicError<Hash: Encode + Decode> {
	/// Extrinsic was not successfully executed onchain.
	FailedExtrinsic(FailedExtrinsicError<Hash>),
	/// Api Error.
	ApiError(Error),
	/// Rpc Client Error.
	RpcClient(RpcClientError),
	/// Could not send update because the Stream has been closed unexpectedly.
	NoStream,
}

/// Encountered unexpected tx status during watch process or the extrinsic failed.
#[derive(Debug)]
pub struct FailedExtrinsicError<Hash: Encode + Decode> {
	pub dispatch_error: DispatchError,
	pub report: ExtrinsicReport<Hash>,
}
