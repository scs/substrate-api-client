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

use primitives::H256 as Hash;
use primitive_types::U256;
use primitives::blake2_256;
use primitives::twox_128;

pub fn storage_key_hash(module: &str, storage_key_name: &str, param: Option<Vec<u8>>) -> String {
    let mut key = [module, storage_key_name].join(" ").as_bytes().to_vec();
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
    let hexstr = hexstr.trim_matches('\"')
        .to_string()
        .trim_start_matches("0x")
        .to_string();

    hex::decode(&hexstr).unwrap()
}

pub fn hexstr_to_u64(hexstr: String) -> u64 {
    let unhex = hexstr_to_vec(hexstr);
    let mut h: [u8; 8] = Default::default();
    h.copy_from_slice(&unhex);
    u64::from_le_bytes(h)
}

pub fn hexstr_to_u256(hexstr: String) -> U256 {
    let _unhex = hexstr_to_vec(hexstr);
    U256::from_little_endian(&_unhex[..])
}

pub fn hexstr_to_hash(hexstr: String) -> Hash {
    let _unhex = hexstr_to_vec(hexstr);
    let mut gh: [u8; 32] = Default::default();
    gh.copy_from_slice(&_unhex);
    Hash::from(gh)
}