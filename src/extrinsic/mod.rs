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

use indices::address::Address;
use node_primitives::{Hash, Index, Signature};
use parity_codec::{Compact, Encode};
use primitive_types::U256;
use primitives::{blake2_256, hexdisplay::HexDisplay};
use primitives::offchain::CryptoKind;
use runtime_primitives::generic::{Era, UncheckedMortalCompactExtrinsic};

use calls::{balance_transfer_fn, BalanceTransfer};
use crypto::AccountKey;

use crate::node_metadata::NodeMetadata;

pub type UncheckedExtrinsic<F> = UncheckedMortalCompactExtrinsic<Address<[u8; 32], u32>, Index, F, Signature>;

#[macro_use]
pub mod calls;
pub mod crypto;


// see https://wiki.parity.io/Extrinsic
pub fn transfer(from: &str, to: &str, amount: U256, index: U256, genesis_hash: Hash, crypto_kind: CryptoKind, node_metadata: NodeMetadata) -> UncheckedExtrinsic<BalanceTransfer> {
	let to = AccountKey::public_from_suri(to, Some(""), crypto_kind);
	let function = balance_transfer_fn(Address::from(to), amount.low_u128(), node_metadata);
	compose_extrinsic::<BalanceTransfer>(from, function, index, genesis_hash, crypto_kind)
}

pub fn compose_extrinsic<F: Encode>(from: &str, function: F, index: U256, genesis_hash: Hash, crypto_kind: CryptoKind) -> UncheckedExtrinsic<F> {
	debug!("using genesis hash: {:?}", genesis_hash);

	let signer = AccountKey::new(from, Some(""), crypto_kind);
	let era = Era::immortal();

	let raw_payload = (Compact(index.low_u64()), function, era, genesis_hash);
	let signature = raw_payload.using_encoded(|payload| if payload.len() > 256 {
		signer.sign(&blake2_256(payload)[..])
	} else {
		debug!("signing {}", HexDisplay::from(&payload));
		signer.sign(payload)
	});

	UncheckedExtrinsic {
		signature: Some((Address::from(signer.public()), signature, index.low_u64().into(), era)),
		function: raw_payload.1,
	}
}

#[macro_export]
macro_rules! compose {
	($node_metadata: expr,
	$genesis_hash: expr,
	$crypto_kind: expr,
	$module: expr,
	$call: expr,
	$nonce: expr,
	$from: expr,
	$($args: expr), + ) => {
		{
			use parity_codec::{Compact, Encode};
			use primitives::{blake2_256, hexdisplay::HexDisplay};
			use indices::address::Address;
			use node_primitives::{Hash, Index, Signature};
			use runtime_primitives::generic::Era;
			use substrate_api_client::extrinsic::{crypto::AccountKey, UncheckedExtrinsic};
			use substrate_api_client::compose_call;

			info!("Composing generic extrinsic for module {:?} and call {:?}", $module, $call);

			let call = compose_call!($node_metadata, $module, $call, $( ($args)), +);
			let signer = AccountKey::new($from, Some(""), $crypto_kind);
			let era = Era::immortal();

			let raw_payload = (Compact($nonce.low_u64()), call, era, $genesis_hash);
			let signature = raw_payload.using_encoded(|payload| if payload.len() > 256 {
				signer.sign(&blake2_256(payload)[..])
			} else {
				debug!("signing {}", HexDisplay::from(&payload));
				signer.sign(payload)
			});

			UncheckedExtrinsic {
				signature: Some((Address::from(signer.public()), signature, $nonce.low_u64().into(), era)),
				function: raw_payload.1,
			}
		}
    };
}

#[cfg(test)]
mod tests {
	use node_primitives::Balance;

	use crate::Api;

	use super::*;

	#[test]
	fn call_from_meta_data_index_equals_imported_call() {
		let node_ip = "127.0.0.1";
		let node_port = "9500";
		let url = format!("{}:{}", node_ip, node_port);
		let balance_module_index = 3u8;
		let balance_transfer_index = 0u8;
		println!("Interacting with node on {}", url);

		let mut api = Api::new(format!("ws://{}", url));
		api.init();

		let amount = Balance::from(42 as u128);
		let to = AccountKey::public_from_suri("//Alice", Some(""), CryptoKind::Sr25519);

		let my_call = ([balance_module_index, balance_transfer_index], Address::<[u8; 32], u32>::from(to.clone()), Compact(amount)).encode();
		let transfer_fn = balance_transfer_fn(Address::<[u8; 32], u32>::from(to.clone()), amount, api.metadata.clone()).encode();
		assert_eq!(my_call, transfer_fn);
	}
}