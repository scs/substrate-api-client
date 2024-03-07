/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG
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

//! Contains stuff to instantiate communication with a substrate node.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(error_in_core))]

extern crate alloc;

use alloc::{borrow::ToOwned, vec::Vec};
use codec::{Decode, Encode};

pub use alloc::{collections::BTreeMap, vec};
pub use events::{EventDetails, Events, RawEventDetails};
pub use metadata::{Metadata, MetadataError};
pub use scale_decode::DecodeAsType;

pub mod error;
pub mod events;
pub mod metadata;
pub mod storage;

#[cfg(any(feature = "mocks", test))]
pub mod test_utils;

/// Wraps an already encoded byte vector, prevents being encoded as a raw byte vector as part of
/// the transaction payload
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Encoded(pub Vec<u8>);

impl codec::Encode for Encoded {
	fn encode(&self) -> Vec<u8> {
		self.0.to_owned()
	}
}

// This following types were taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/

/// Trait to uniquely identify the events's identity from the runtime metadata.
///
/// Generated API structures that represent an event implement this trait.
///
/// The trait is utilized to decode emitted events from a block, via obtaining the
/// form of the `Event` from the metadata.
pub trait StaticEvent: Decode {
	/// Pallet name.
	const PALLET: &'static str;
	/// Event name.
	const EVENT: &'static str;

	/// Returns true if the given pallet and event names match this event.
	fn is_event(pallet: &str, event: &str) -> bool {
		Self::PALLET == pallet && Self::EVENT == event
	}
}

/// A phase of a block's execution.
// https://github.com/paritytech/substrate/blob/2bfc1dd66ef32cf8beb90007dfb544a9d28f1b2f/frame/system/src/lib.rs#L698-L708
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Encode, Decode)]
pub enum Phase {
	/// Applying an extrinsic.
	ApplyExtrinsic(u32),
	/// Finalizing the block.
	Finalization,
	/// Initializing the block.
	Initialization,
}

/// Record of an event happening.
// https://github.com/paritytech/substrate/blob/2bfc1dd66ef32cf8beb90007dfb544a9d28f1b2f/frame/system/src/lib.rs#L716-L726
#[derive(Encode, Decode, PartialEq, Eq, Clone, Debug)]
pub struct EventRecord<E, T> {
	/// The phase of the block it happened in.
	pub phase: Phase,
	/// The event itself.
	pub event: E,
	/// The list of the topics this event has.
	pub topics: Vec<T>,
}
