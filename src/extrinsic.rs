// Copyright 2019 Supercomputing Systems AG
//
// Partial Authorship Parity Technologies (UK) Ltd.
// This file is derived from Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.



use primitives::{ed25519, sr25519, hexdisplay::HexDisplay, Pair, crypto::Ss58Codec, blake2_256};
use runtime_primitives::generic::Era;
use node_runtime::{UncheckedExtrinsic, CheckedExtrinsic, Call, BalancesCall};

use parity_codec::{Encode, Compact};
use node_primitives::{Balance, Index, Hash};

use substrate_bip39::mini_secret_from_entropy;
use bip39::{Mnemonic, Language, MnemonicType};
use rand::{RngCore, rngs::OsRng};
use schnorrkel::keys::MiniSecretKey;
use primitive_types::U256;


trait Crypto {
	type Seed: AsRef<[u8]> + AsMut<[u8]> + Sized + Default;
	type Pair: Pair;
	fn generate_phrase() -> String {
		Mnemonic::new(MnemonicType::Words12, Language::English).phrase().to_owned()
	}
	fn generate_seed() -> Self::Seed {
		let mut seed: Self::Seed = Default::default();
		OsRng::new().unwrap().fill_bytes(seed.as_mut());
		seed
	}
	fn seed_from_phrase(phrase: &str, password: Option<&str>) -> Self::Seed;
	fn pair_from_seed(seed: &Self::Seed) -> Self::Pair;
	fn pair_from_suri(phrase: &str, password: Option<&str>) -> Self::Pair {
		Self::pair_from_seed(&Self::seed_from_phrase(phrase, password))
	}
	fn ss58_from_pair(pair: &Self::Pair) -> String;
	fn public_from_pair(pair: &Self::Pair) -> Vec<u8>;
	fn seed_from_pair(_pair: &Self::Pair) -> Option<&Self::Seed> { None }
	fn print_from_seed(seed: &Self::Seed) {
		let pair = Self::pair_from_seed(seed);
		println!("Seed 0x{} is account:\n  Public key (hex): 0x{}\n  Address (SS58): {}",
			HexDisplay::from(&seed.as_ref()),
			HexDisplay::from(&Self::public_from_pair(&pair)),
			Self::ss58_from_pair(&pair)
		);
	}
	fn print_from_phrase(phrase: &str, password: Option<&str>) {
		let seed = Self::seed_from_phrase(phrase, password);
		let pair = Self::pair_from_seed(&seed);
		println!("Phrase `{}` is account:\n  Seed: 0x{}\n  Public key (hex): 0x{}\n  Address (SS58): {}",
			phrase,
			HexDisplay::from(&seed.as_ref()),
			HexDisplay::from(&Self::public_from_pair(&pair)),
			Self::ss58_from_pair(&pair)
		);
	}
	fn print_from_uri(uri: &str, password: Option<&str>) where <Self::Pair as Pair>::Public: Sized + Ss58Codec + AsRef<[u8]> {
		if let Ok(pair) = Self::Pair::from_string(uri, password) {
			let seed_text = Self::seed_from_pair(&pair)
				.map_or_else(Default::default, |s| format!("\n  Seed: 0x{}", HexDisplay::from(&s.as_ref())));
			println!("Secret Key URI `{}` is account:{}\n  Public key (hex): 0x{}\n  Address (SS58): {}",
				uri,
				seed_text,
				HexDisplay::from(&Self::public_from_pair(&pair)),
				Self::ss58_from_pair(&pair)
			);
		}
		if let Ok(public) = <Self::Pair as Pair>::Public::from_string(uri) {
			println!("Public Key URI `{}` is account:\n  Public key (hex): 0x{}\n  Address (SS58): {}",
				uri,
				HexDisplay::from(&public.as_ref()),
				public.to_ss58check()
			);
		}
	}
}


struct Ed25519;

impl Crypto for Ed25519 {
	type Seed = [u8; 32];
	type Pair = ed25519::Pair;

	fn seed_from_phrase(phrase: &str, password: Option<&str>) -> Self::Seed {
		Sr25519::seed_from_phrase(phrase, password)
	}
	fn pair_from_suri(suri: &str, password_override: Option<&str>) -> Self::Pair {
		ed25519::Pair::from_legacy_string(suri, password_override)
	}
	fn pair_from_seed(seed: &Self::Seed) -> Self::Pair { ed25519::Pair::from_seed(seed.clone()) }
	fn ss58_from_pair(pair: &Self::Pair) -> String { pair.public().to_ss58check() }
	fn public_from_pair(pair: &Self::Pair) -> Vec<u8> { (&pair.public().0[..]).to_owned() }
	fn seed_from_pair(pair: &Self::Pair) -> Option<&Self::Seed> { Some(pair.seed()) }
}

struct Sr25519;

impl Crypto for Sr25519 {
	type Seed = [u8; 32];
	type Pair = sr25519::Pair;

	fn seed_from_phrase(phrase: &str, password: Option<&str>) -> Self::Seed {
		mini_secret_from_entropy(
			Mnemonic::from_phrase(phrase, Language::English)
				.unwrap_or_else(|_|
					panic!("Phrase is not a valid BIP-39 phrase: \n    {}", phrase)
				)
				.entropy(),
			password.unwrap_or("")
		)
			.expect("32 bytes can always build a key; qed")
			.to_bytes()
	}

	fn pair_from_suri(suri: &str, password: Option<&str>) -> Self::Pair {
		sr25519::Pair::from_string(suri, password).expect("Invalid phrase")
	}

	fn pair_from_seed(seed: &Self::Seed) -> Self::Pair {
		MiniSecretKey::from_bytes(seed)
			.expect("32 bytes can always build a key; qed")
			.into()
	}
	fn ss58_from_pair(pair: &Self::Pair) -> String { pair.public().to_ss58check() }
	fn public_from_pair(pair: &Self::Pair) -> Vec<u8> { (&pair.public().0[..]).to_owned() }
}

fn sign(xt: CheckedExtrinsic, key: &sr25519::Pair, genesis_hash: Hash) -> UncheckedExtrinsic {
	use parity_codec::Encode;
	match xt.signed {
		Some((signed, index)) => {
			let era = Era::immortal();
			let payload = (index.into(), xt.function, era, genesis_hash);
			assert_eq!(key.public(), signed);
			let signature = payload.using_encoded(|b| {
				if b.len() > 256 {
					key.sign(&blake2_256(b))
				} else {
					key.sign(b)
				}
			}).into();
			UncheckedExtrinsic {
				signature: Some((signed.into(), signature, payload.0, era)),
				function: payload.1,
			}
		}
		None => UncheckedExtrinsic {
			signature: None,
			function: xt.function,
		},
	}
}


// see https://wiki.parity.io/Extrinsic
pub fn transfer(from: &str, to: &str, amount: U256, index: U256, genesis_hash: Hash) -> UncheckedExtrinsic {
		let signer = Sr25519::pair_from_suri(from, Some(""));

		let to = sr25519::Public::from_string(to).ok().or_else(||
			sr25519::Pair::from_string(to, Some("")).ok().map(|p| p.public())
		).expect("Invalid 'to' URI; expecting either a secret URI or a public URI.");
		let amount = Balance::from(amount.low_u128());
		let index = Index::from(index.low_u64());
		//let amount = str::parse::<Balance>("42")
		//	.expect("Invalid 'amount' parameter; expecting an integer.");
		//let index = str::parse::<Index>("0")
		//	.expect("Invalid 'index' parameter; expecting an integer.");

		let function = Call::Balances(BalancesCall::transfer(to.into(), amount));

		let era = Era::immortal();

		println!("using genesis hash: {:?}", genesis_hash);
/*		let mut gh: [u8; 32] = Default::default();
    	gh.copy_from_slice(hex::decode(genesis_hash).unwrap().as_ref());
		let genesis_hash = Hash::from(gh);
		println!("using genesis hash to Hash: {:?}", gh);
*/
		//let genesis_hash: Hash = hex::decode(genesis_hash).unwrap();
		//let genesis_hash: Hash = hex!["61b81c075e1e54b17a2f2d685a3075d3e5f5c7934456dd95332e68dd751a4b40"].into();
//			let genesis_hash: Hash = hex!["58afaad82f5a80ecdc8e974f5d88c4298947260fb05e34f84a9eed18ec5a78f9"].into();
		let raw_payload = (Compact(index), function, era, genesis_hash);
		let signature = raw_payload.using_encoded(|payload| if payload.len() > 256 {
			signer.sign(&blake2_256(payload)[..])
		} else {
			println!("signing {}", HexDisplay::from(&payload));
			signer.sign(payload)
		});
		UncheckedExtrinsic::new_signed(
			index,
			raw_payload.1,
			signer.public().into(),
			signature.into(),
			era,
		)
	}

