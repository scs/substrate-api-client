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
use alloc::{boxed::Box, vec::Vec};
use codec::{Decode, Encode};

#[cfg(not(feature = "std"))]
use core::error::Error as ErrorT;
#[cfg(feature = "std")]
use std::error::Error as ErrorT;

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
	/// Extrinsic failed onchain. Contains the encoded report and the associated dispatch error.
	FailedExtrinsic(FailedExtrinsicError),
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
	Other(Box<dyn ErrorT + Send + Sync + 'static>),
}

/// Encountered unexpected tx status during watch process or the extrinsic failed.
#[derive(Debug)]
pub struct FailedExtrinsicError {
	dispatch_error: DispatchError,
	encoded_report: Vec<u8>,
}

impl FailedExtrinsicError {
	pub fn new(dispatch_error: DispatchError, encoded_report: Vec<u8>) -> Self {
		Self { dispatch_error, encoded_report }
	}

	pub fn dispatch_error(&self) -> &DispatchError {
		&self.dispatch_error
	}

	pub fn get_report<Hash: Encode + Decode>(&self) -> Result<ExtrinsicReport<Hash>> {
		let report = Decode::decode(&mut self.encoded_report.as_slice())?;
		Ok(report)
	}

	pub fn encoded_report(&self) -> &[u8] {
		&self.encoded_report
	}
}
