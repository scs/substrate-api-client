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

use ac_primitives::StorageKey;
use alloc::{string::String, vec::Vec};
use hex::FromHexError;
use sp_core::{twox_128, H256};

pub fn storage_key(module: &str, storage_key_name: &str) -> StorageKey {
	let mut key = twox_128(module.as_bytes()).to_vec();
	key.extend(twox_128(storage_key_name.as_bytes()));
	StorageKey(key)
}

pub trait FromHexString {
	fn from_hex(hex: String) -> Result<Self, hex::FromHexError>
	where
		Self: Sized;
}

impl FromHexString for Vec<u8> {
	fn from_hex(hex: String) -> Result<Self, hex::FromHexError> {
		let hexstr = hex.trim_matches('\"').trim_start_matches("0x");

		hex::decode(hexstr)
	}
}

impl FromHexString for H256 {
	fn from_hex(hex: String) -> Result<Self, FromHexError> {
		let vec = Vec::from_hex(hex)?;

		match vec.len() {
			32 => Ok(H256::from_slice(&vec)),
			_ => Err(hex::FromHexError::InvalidStringLength),
		}
	}
}

pub trait ToHexString {
	fn to_hex(self) -> String
	where
		Self: Sized;
}

impl ToHexString for Vec<u8> {
	fn to_hex(self) -> String {
		let mut hex_str = hex::encode(self);
		hex_str.insert_str(0, "0x");
		hex_str
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_hextstr_to_vec() {
		assert_eq!(Vec::from_hex("0x01020a".to_string()), Ok(vec!(1, 2, 10)));
		assert_eq!(
			Vec::from_hex("null".to_string()),
			Err(hex::FromHexError::InvalidHexCharacter { c: 'n', index: 0 })
		);
		assert_eq!(
			Vec::from_hex("0x0q".to_string()),
			Err(hex::FromHexError::InvalidHexCharacter { c: 'q', index: 1 })
		);
	}

	#[test]
	fn test_hextstr_to_hash() {
		assert_eq!(
			H256::from_hex(
				"0x0000000000000000000000000000000000000000000000000000000000000000".to_string()
			),
			Ok(H256::from([0u8; 32]))
		);
		assert_eq!(
			H256::from_hex("0x010000000000000000".to_string()),
			Err(hex::FromHexError::InvalidStringLength)
		);
		assert_eq!(
			H256::from_hex("0x0q".to_string()),
			Err(hex::FromHexError::InvalidHexCharacter { c: 'q', index: 1 })
		);
	}
}
