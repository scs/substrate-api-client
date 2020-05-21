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

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{string::String, string::ToString, vec::Vec};

use hex::FromHexError;
use sp_core::storage::StorageKey;
use sp_core::twox_128;
use sp_core::H256 as Hash;

pub fn storage_key(module: &str, storage_key_name: &str) -> StorageKey {
    let mut key = twox_128(module.as_bytes()).to_vec();
    key.extend(&twox_128(storage_key_name.as_bytes()));
    StorageKey(key)
}

pub fn hexstr_to_vec(hexstr: String) -> Result<Vec<u8>, FromHexError> {
    let hexstr = hexstr
        .trim_matches('\"')
        .to_string()
        .trim_start_matches("0x")
        .to_string();

    hex::decode(&hexstr)
}

pub fn hexstr_to_hash(hexstr: String) -> Result<Hash, FromHexError> {
    let unhex = hexstr_to_vec(hexstr);
    match unhex {
        Ok(vec) => match vec.len() {
            32 => {
                let mut gh: [u8; 32] = Default::default();
                gh.copy_from_slice(&vec[..]);
                Ok(Hash::from(gh))
            }
            _ => Err(hex::FromHexError::InvalidStringLength),
        },
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_hextstr_to_vec() {
        assert_eq!(hexstr_to_vec("0x01020a".to_string()), Ok(vec!(1, 2, 10)));
        assert_eq!(
            hexstr_to_vec("null".to_string()),
            Err(hex::FromHexError::InvalidHexCharacter { c: 'n', index: 0 })
        );
        assert_eq!(
            hexstr_to_vec("0x0q".to_string()),
            Err(hex::FromHexError::InvalidHexCharacter { c: 'q', index: 1 })
        );
    }

    #[test]
    fn test_hextstr_to_hash() {
        assert_eq!(
            hexstr_to_hash(
                "0x0000000000000000000000000000000000000000000000000000000000000000".to_string()
            ),
            Ok(Hash::from([0u8; 32]))
        );
        assert_eq!(
            hexstr_to_hash("0x010000000000000000".to_string()),
            Err(hex::FromHexError::InvalidStringLength)
        );
        assert_eq!(
            hexstr_to_hash("0x0q".to_string()),
            Err(hex::FromHexError::InvalidHexCharacter { c: 'q', index: 1 })
        );
    }
}
