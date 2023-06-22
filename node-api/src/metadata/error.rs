/*
	Copyright 2021 Supercomputing Systems AG
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

use alloc::string::String;
use codec::{Decode, Encode, Error as CodecError};

/// Metadata error originated from inspecting the internal representation of the runtime metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataError {
	/// Module is not in metadata.
	PalletNotFound(String),
	/// Pallet is not in metadata.
	PalletIndexNotFound(u8),
	/// Call is not in metadata.
	CallNotFound(&'static str),
	/// Event is not in metadata.
	EventNotFound(u8, u8),
	/// Error is not in metadata.
	ErrorNotFound(u8, u8),
	/// Storage is not in metadata.
	StorageNotFound(&'static str),
	/// Storage type does not match requested type.
	StorageTypeError,
	/// Default error.
	DefaultError(CodecError),
	/// Failure to decode constant value.
	ConstantValueError(CodecError),
	/// Constant is not in metadata.
	ConstantNotFound(&'static str),
	/// Type is not in metadata.
	TypeNotFound(u32),
	/// Runtime constant metadata is incompatible with the static one.
	IncompatibleConstantMetadata(String, String),
	/// Runtime call metadata is incompatible with the static one.
	IncompatibleCallMetadata(String, String),
	/// Runtime storage metadata is incompatible with the static one.
	IncompatibleStorageMetadata(String, String),
	/// Runtime metadata is not fully compatible with the static one.
	IncompatibleMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Encode, Decode)]
pub enum InvalidMetadataError {
	InvalidPrefix,
	InvalidVersion,
	/// Type is missing from type registry.
	MissingType(u32),
	/// Type was not variant/enum type.
	TypeDefNotVariant(u32),
}
