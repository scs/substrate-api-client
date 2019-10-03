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


use rstd::prelude::*;
use primitives::{crypto::Ss58Codec, ed25519, sr25519, Pair};
use runtime_primitives::AnySignature as Signature;

pub enum CryptoKind {
    Ed25519,
    Sr25519,
}

#[derive(Clone)]
pub enum AccountKey {
    Ed(ed25519::Pair),
    Sr(sr25519::Pair),
}


impl AccountKey {
    #[cfg(feature = "std")]
    pub fn new(phrase: &str, password: Option<&str>, kind: CryptoKind) -> AccountKey {
        match kind {
            CryptoKind::Ed25519 => {
                let pair = Ed25519::pair_from_suri(phrase, password);
                AccountKey::Ed(pair)
            },
            CryptoKind::Sr25519 => {
                let pair = Sr25519::pair_from_suri(phrase, password);
                AccountKey::Sr(pair)
            }
        }
    }
    pub fn public(&self) -> [u8; 32] {
        match self {
            AccountKey::Ed(pair) => pair.public().0,
            AccountKey::Sr(pair) => pair.public().0,
        }
    }
    #[cfg(feature = "std")]
    pub fn public_from_suri(phrase: &str, password: Option<&str>, kind: CryptoKind) -> [u8; 32] {
        match kind {
            CryptoKind::Ed25519 => Ed25519::public_from_suri(phrase, password).0,
            CryptoKind::Sr25519 => Sr25519::public_from_suri(phrase, password).0,
        }
    }

    pub fn sign(&self, raw_payload: &[u8]) -> Signature {
        match self {
            AccountKey::Ed(pair) => pair.sign(raw_payload).into(),
            AccountKey::Sr(pair) => pair.sign(raw_payload).into(),
        }
    }
}

pub struct Sr25519;

pub trait Crypto {
    type Seed: AsRef<[u8]> + AsMut<[u8]> + Sized + Default;
    type Pair: Pair;
    
    #[cfg(feature = "std")]
    type Public: Ss58Codec + AsRef<[u8]> + core::hash::Hash;
    #[cfg(not(feature = "std"))]
    type Public: AsRef<[u8]> + core::hash::Hash;

    #[cfg(feature = "std")]
    fn pair_from_suri(phrase: &str, password: Option<&str>) -> Self::Pair;
    #[cfg(feature = "std")]
    fn public_from_suri(phrase: &str, password: Option<&str>) -> Self::Public;
    #[cfg(feature = "std")]
    fn ss58_from_pair(pair: &Self::Pair) -> String;
}

impl Crypto for Sr25519 {
    type Seed = [u8; 32];
    type Pair = sr25519::Pair;
    type Public = sr25519::Public;

    #[cfg(feature = "std")]
    fn pair_from_suri(suri: &str, password: Option<&str>) -> Self::Pair {
        sr25519::Pair::from_string(suri, password).expect("Invalid phrase")
    }

    #[cfg(feature = "std")]
    fn public_from_suri(suri: &str, password: Option<&str>) -> Self::Public {
        sr25519::Public::from_string(suri)
            .ok()
            .or_else(|| {
                sr25519::Pair::from_string(suri, password)
                    .ok()
                    .map(|p| p.public())
            })
            .expect("Invalid 'to' URI; expecting either a secret URI or a public URI.")
    }

    #[cfg(feature = "std")]
    fn ss58_from_pair(pair: &Self::Pair) -> String { pair.public().to_ss58check() }

}

struct Ed25519;

impl Crypto for Ed25519 {
    type Seed = [u8; 32];
    type Pair = ed25519::Pair;
    type Public = ed25519::Public;

    #[cfg(feature = "std")]
    fn pair_from_suri(suri: &str, password_override: Option<&str>) -> Self::Pair {
        ed25519::Pair::from_string(suri, password_override)
            .expect("Invalid 'to' URI; expecting either a secret URI or a public URI.")
    }
    #[cfg(feature = "std")]
    fn public_from_suri(suri: &str, password_override: Option<&str>) -> Self::Public {
        ed25519::Public::from_string(suri)
            .ok()
            .or_else(|| {
                ed25519::Pair::from_string(suri, password_override)
                    .ok()
                    .map(|p| p.public())
            })
            .expect("Invalid 'to' URI; expecting either a secret URI or a public URI.")
    }
    #[cfg(feature = "std")]
    fn ss58_from_pair(pair: &Self::Pair) -> String {
        pair.public().to_ss58check()
    }
}
