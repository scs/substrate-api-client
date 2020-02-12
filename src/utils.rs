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

use hex::FromHexError;
use primitive_types::U256;
use sp_core::blake2_256;
use sp_core::twox_128;
use sp_core::H256 as Hash;

fn storage_key_hash_vec(module: &str, storage_key_name: &str, param: Option<Vec<u8>>) -> Vec<u8> {
    let mut key = [module, storage_key_name].join(" ").as_bytes().to_vec();
    match param {
        Some(par) => {
            key.extend(&par);
            blake2_256(&key).to_vec()
        }
        _ => twox_128(&key).to_vec(),
    }
}

pub fn storage_key_hash(module: &str, storage_key_name: &str, param: Option<Vec<u8>>) -> String {
    let mut keyhash_str = hex::encode(storage_key_hash_vec(module, storage_key_name, param));
    keyhash_str.insert_str(0, "0x");
    keyhash_str
}

pub fn storage_key_hash_double_map(
    module: &str,
    storage_key_name: &str,
    first: Vec<u8>,
    second: Vec<u8>,
) -> String {
    let mut keyhash = storage_key_hash_vec(module, storage_key_name, Some(first));
    keyhash.extend(&blake2_256(&second).to_vec());
    let mut keyhash_str = hex::encode(keyhash);
    keyhash_str.insert_str(0, "0x");
    keyhash_str
}

pub fn hexstr_to_vec(hexstr: String) -> Result<Vec<u8>, FromHexError> {
    let hexstr = hexstr
        .trim_matches('\"')
        .to_string()
        .trim_start_matches("0x")
        .to_string();
    match hexstr.as_str() {
        "null" => Ok(vec![0u8]),
        _ => hex::decode(&hexstr),
    }
}

pub fn hexstr_to_u64(hexstr: String) -> Result<u64, FromHexError> {
    let unhex = hexstr_to_vec(hexstr);
    match unhex {
        Ok(vec) => match vec.len() {
            1 | 2 | 4 | 8 => {
                let mut h: [u8; 8] = Default::default();
                h[..vec.len()].copy_from_slice(&vec);
                Ok(u64::from_le_bytes(h))
            }
            _ => match vec.iter().sum() {
                0 => Ok(0u64),
                _ => Err(hex::FromHexError::InvalidStringLength),
            },
        },
        Err(err) => Err(err),
    }
}

pub fn hexstr_to_u256(hexstr: String) -> Result<U256, FromHexError> {
    let unhex = hexstr_to_vec(hexstr);
    match unhex {
        Ok(vec) => match vec.len() {
            1 | 2 | 4 | 8 | 16 | 32 => Ok(U256::from_little_endian(&vec[..])),
            _ => match vec.iter().sum() {
                0 => Ok(U256::from(0)),
                _ => Err(hex::FromHexError::InvalidStringLength),
            },
        },
        Err(err) => Err(err),
    }
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
        assert_eq!(hexstr_to_vec("null".to_string()), Ok(vec!(0u8)));
        assert_eq!(
            hexstr_to_vec("0x0q".to_string()),
            Err(hex::FromHexError::InvalidHexCharacter { c: 'q', index: 1 })
        );
    }

    #[test]
    fn test_hextstr_to_u64() {
        assert_eq!(hexstr_to_u64("0x0100000000000000".to_string()), Ok(1u64));
        assert_eq!(hexstr_to_u64("0x01000000".to_string()), Ok(1u64));
        assert_eq!(hexstr_to_u64("null".to_string()), Ok(0u64));
        assert_eq!(
            hexstr_to_u64("0x010000000000000000".to_string()),
            Err(hex::FromHexError::InvalidStringLength)
        );
        assert_eq!(
            hexstr_to_u64("0x0q".to_string()),
            Err(hex::FromHexError::InvalidHexCharacter { c: 'q', index: 1 })
        );
    }

    #[test]
    fn test_hextstr_to_u256() {
        assert_eq!(
            hexstr_to_u256(
                "0x0100000000000000000000000000000000000000000000000000000000000000".to_string()
            ),
            Ok(U256::from(1))
        );
        assert_eq!(hexstr_to_u256("0x01000000".to_string()), Ok(U256::from(1)));
        assert_eq!(hexstr_to_u256("null".to_string()), Ok(U256::from(0)));
        assert_eq!(
            hexstr_to_u256("0x010000000000000000".to_string()),
            Err(hex::FromHexError::InvalidStringLength)
        );
        assert_eq!(
            hexstr_to_u256("0x0q".to_string()),
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
