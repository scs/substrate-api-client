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

use node_primitives::Hash;
use primitive_types::U256;
use primitives::blake2_256;
use primitives::twox_128;

pub fn storage_key_hash(module: &str, storage_key_name: &str, param: Option<Vec<u8>>) -> String {
    let mut key = module.as_bytes().to_vec();
    key.append(&mut vec!(' ' as u8));
    key.append(&mut storage_key_name.as_bytes().to_vec());
    let mut keyhash;
    match param {
        Some(par) => {
            key.append(&mut par.clone());
            keyhash = hex::encode(blake2_256(&key));
        },
        _ => {
            keyhash = hex::encode(twox_128(&key));
        },
    }
    keyhash.insert_str(0, "0x");
    keyhash
}

pub fn hexstr_to_vec(hexstr: String) -> Vec<u8> {
    let mut _hexstr = hexstr.clone();
    if _hexstr.starts_with("0x") {
        _hexstr.remove(0);
        _hexstr.remove(0);
    } else {
        info!("converting non-prefixed hex string")
    }
    hex::decode(&_hexstr).unwrap()
}

pub fn hexstr_to_u256(hexstr: String) -> U256 {
    let _unhex = hexstr_to_vec(hexstr);
    U256::from_little_endian(&mut &_unhex[..])
}

pub fn hexstr_to_hash(hexstr: String) -> Hash {
    let _unhex = hexstr_to_vec(hexstr);
    let mut gh: [u8; 32] = Default::default();
    gh.copy_from_slice(&_unhex);
    Hash::from(gh)
}