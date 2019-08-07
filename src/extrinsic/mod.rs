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
use primitives::offchain::CryptoKind;

use definitions::*;
use crypto::AccountKey;

use crate::node_metadata::NodeMetadata;

pub mod definitions;
pub mod crypto;

#[macro_export]
macro_rules! compose_call {
    ($node_metadata: expr, $module: expr, $call_name: expr, $($args: expr),+ ) => {
        {
            let mut metad = $node_metadata;
            metad.retain(|m| !m.calls.is_empty());

            let module_index = metad
            .iter().position( | m | m.name == $module).unwrap();

            let call_index = metad[module_index].calls
            .iter().position( | c| c.name == $call_name).unwrap();

            ([module_index as u8, call_index as u8], $( ($args)), +)
        }
    };
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
			use runtime_primitives::generic::Era;
			use crate::extrinsic::{crypto::AccountKey, definitions::UncheckedExtrinsic};

			info!("Composing generic extrinsic for module {:?} and call {:?}", $module, $call);

			let call = $crate::compose_call!($node_metadata, $module, $call, $( ($args)), +);
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

pub fn transfer(from: &str, to: &str, amount: u128, index: U256, genesis_hash: Hash, crypto_kind: CryptoKind, node_metadata: NodeMetadata) -> UncheckedExtrinsic<BalanceTransfer> {
	let to = AccountKey::public_from_suri(to, Some(""), crypto_kind);
	compose!(node_metadata, genesis_hash, crypto_kind, BALANCES_MODULE_NAME, BALANCES_TRANSFER, index, from, Address::from(to), Compact(amount))
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