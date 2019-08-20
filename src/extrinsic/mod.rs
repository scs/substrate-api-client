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

pub mod xt_primitives;
pub mod contract;
pub mod balances;

#[macro_export]
macro_rules! compose_call {
    ($node_metadata: expr, $module: expr, $call_name: expr, $($args: expr),+ ) => {
        {
            let mut meta = $node_metadata;
            meta.retain(|m| !m.calls.is_empty());

            let module_index = meta
            .iter().position(|m| m.name == $module).expect("Module not found in Metadata");

            let call_index = meta[module_index].calls
            .iter().position(|c| c.name == $call_name).expect("Call not found in Module");

            ([module_index as u8, call_index as u8], $(($args)), +)
        }
    };
}

/// Macro that generates a Unchecked extrinsic for a given module and call passed as a String.
#[macro_export]
macro_rules! compose_extrinsic {
	($node_metadata: expr,
	$genesis_hash: expr,
	$module: expr,
	$call: expr,
	$extra: expr,
	$from: expr,
	$($args: expr), * ) => {
		{
			use codec::{Compact, Encode};
			use primitives::{blake2_256, hexdisplay::HexDisplay};
			use crate::extrinsic::xt_primitives::*;

			info!("Composing generic extrinsic for module {:?} and call {:?}", $module, $call);

			let call = $crate::compose_call!($node_metadata, $module, $call, $(($args)), +);

			let raw_payload = (call, $extra, ($genesis_hash, $genesis_hash));
			let signature = raw_payload.using_encoded(|payload| if payload.len() > 256 {
				$from.sign(&blake2_256(payload)[..])
			} else {
				debug!("signing {}", HexDisplay::from(&payload));
				$from.sign(payload)
			});

			UncheckedExtrinsicV3::new_signed(
				raw_payload.0, GenericAddress::from($from.public()), signature, $extra
			)
		}
    };
}

#[cfg(test)]
mod tests {
	use codec::{Compact, Encode};
	use node_primitives::Balance;

	use xt_primitives::*;

	use crate::Api;
	use crate::crypto::*;
	use crate::extrinsic::balances::{BALANCES_MODULE, BALANCES_TRANSFER};

	use super::*;

	fn test_api() -> Api {
		let node_ip = "127.0.0.1";
		let node_port = "9500";
		let url = format!("{}:{}", node_ip, node_port);
		println!("Interacting with node on {}", url);
		Api::new(format!("ws://{}", url))
	}

	#[test]
	fn call_from_meta_data_works() {
		let api = test_api();

		let balance_module_index = 3u8;
		let balance_transfer_index = 0u8;

		let amount = Balance::from(42 as u128);
		let to = AccountKey::public_from_suri("//Alice", Some(""), CryptoKind::Sr25519);

		let my_call = ([balance_module_index, balance_transfer_index], GenericAddress::from(to.clone()), Compact(amount)).encode();
		let transfer_fn = compose_call!(api.metadata.clone(), BALANCES_MODULE, BALANCES_TRANSFER, GenericAddress::from(to), Compact(amount)).encode();
		assert_eq!(my_call, transfer_fn);
	}
}