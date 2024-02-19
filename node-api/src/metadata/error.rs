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
use codec::{Decode, Encode};

/// Metadata error originated from inspecting the internal representation of the runtime metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataError {
	/// The DispatchError type isn't available in the metadata.
	DispatchErrorNotFound,
	/// Module is not in metadata.
	PalletNameNotFound(String),
	/// Pallet is not in metadata.
	PalletIndexNotFound(u8),
	/// Event type not found in metadata.
	EventTypeNotFoundInPallet(u8),
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
	/// Constant is not in metadata.
	ConstantNotFound(&'static str),
	/// Variant not found.
	VariantIndexNotFound(u8),
	/// Api is not in metadata.
	RuntimeApiNotFound(String),
	/// Exptected a different type of Metadata. Has there been a runtime upgrade inbetween?
	MetadataMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Encode, Decode)]
pub enum MetadataConversionError {
	InvalidPrefix,
	InvalidVersion,
	/// Type is missing from type registry.
	MissingType(u32),
	/// Type was not variant/enum type.
	TypeDefNotVariant(u32),
	/// Type is not in metadata.
	TypeNotFound(u32),
	/// Type Name is not in metadata.
	TypeNameNotFound(String),
	// Path not found.
	InvalidTypePath(String),
}
