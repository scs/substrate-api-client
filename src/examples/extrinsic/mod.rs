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


// This module depends on node_runtime.
// To avoid dependency collisions, node_runtime has been removed from the substrate-api-client library.
//
// Replace this crate by your own if you run a custom substrate node

use node_primitives::{Balance, Hash, Index};
use node_runtime::{BalancesCall, Call, UncheckedExtrinsic};
use parity_codec::{Compact, Encode};
use primitive_types::U256;
use primitives::{/*ed25519, */blake2_256, crypto::Ss58Codec, hexdisplay::HexDisplay, Pair, sr25519};
use runtime_primitives::generic::Era;

use crypto::{Crypto, Sr25519};

mod crypto;

// see https://wiki.parity.io/Extrinsic
pub fn transfer(from: &str, to: &str, amount: U256, index: U256, genesis_hash: Hash) -> UncheckedExtrinsic {
		let signer = Sr25519::pair_from_suri(from, Some(""));

		let to = sr25519::Public::from_string(to).ok().or_else(||
			sr25519::Pair::from_string(to, Some("")).ok().map(|p| p.public())
		).expect("Invalid 'to' URI; expecting either a secret URI or a public URI.");
		let amount = Balance::from(amount.low_u128());
		let index = Index::from(index.low_u64());

		let function = Call::Balances(BalancesCall::transfer(to.into(), amount));

		let era = Era::immortal();

		debug!("using genesis hash: {:?}", genesis_hash);
		let raw_payload = (Compact(index), function, era, genesis_hash);
		let signature = raw_payload.using_encoded(|payload| if payload.len() > 256 {
			signer.sign(&blake2_256(payload)[..])
		} else {
			debug!("signing {}", HexDisplay::from(&payload));
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

// pub fn sign(xt: CheckedExtrinsic, key: &sr25519::Pair, genesis_hash: Hash) -> UncheckedExtrinsic {
// 	match xt.signed {
// 		Some((signed, index)) => {
// 			let era = Era::immortal();
// 			let payload = (index.into(), xt.function, era, genesis_hash);
// 			assert_eq!(key.public(), signed);
// 			let signature = payload.using_encoded(|b| {
// 				if b.len() > 256 {
// 					key.sign(&blake2_256(b))
// 				} else {
// 					key.sign(b)
// 				}
// 			}).into();
// 			UncheckedExtrinsic {
// 				signature: Some((signed.into(), signature, payload.0, era)),
// 				function: payload.1,
// 			}
// 		}
// 		None => UncheckedExtrinsic {
// 			signature: None,
// 			function: xt.function,
// 		},
// 	}
// }

