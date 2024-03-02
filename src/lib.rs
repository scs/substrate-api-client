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
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(error_in_core))]

extern crate alloc;

use sp_crypto_hashing::twox_128;
use sp_storage::StorageKey;

pub use ac_compose_macros;
pub use ac_node_api;
pub use ac_primitives;
pub use api::*; // Re-export everything

pub mod api;
pub mod extrinsic;
pub mod rpc;

/// Returns the concatenated 128 bit hash of the given module and specific storage key
/// as a full Substrate StorageKey.
pub fn storage_key(module: &str, storage_key_name: &str) -> StorageKey {
	let mut key = twox_128(module.as_bytes()).to_vec();
	key.extend(twox_128(storage_key_name.as_bytes()));
	StorageKey(key)
}
