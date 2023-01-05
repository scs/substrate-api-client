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

#[derive(Debug, derive_more::From)]
pub enum Error {
	FetchGenesisHash,
	NoSigner,
	RpcClient(RpcClientError),
	Metadata(MetadataError),
	InvalidMetadata(InvalidMetadataError),
	NodeApi(ac_node_api::error::Error),
	StorageValueDecode(codec::Error),
	UnsupportedXtStatus(XtStatus),
	TryFromIntError,
	Dispatch(DispatchError),
	Extrinsic(String),
	NoStream,
	NoBlockHash,
	NoBlock,
	Other(Box<dyn core::error::Error + Send + Sync + 'static>),
}
