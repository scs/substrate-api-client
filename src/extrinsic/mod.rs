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

use definitions::*;
use crypto::AccountKey;

use crate::node_metadata::NodeMetadata;

pub mod definitions;
pub mod crypto;

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
	$nonce: expr,
	$from: expr,
	$($args: expr), * ) => {
		{
			use codec::{Compact, Encode};
			use primitives::{blake2_256, hexdisplay::HexDisplay};
			use indices::address::Address;
			use runtime_primitives::generic::Era;
			use crate::extrinsic::definitions::*;

			info!("Composing generic extrinsic for module {:?} and call {:?}", $module, $call);

			let call = $crate::compose_call!($node_metadata, $module, $call, $(($args)), +);
			let era = Era::immortal();
			let extra = GenericExtra { era: era, nonce: $nonce.low_u64(), tip: 0};

			let raw_payload = (call, extra.clone(), ($genesis_hash, $genesis_hash));
//			let raw_payload = (Compact($nonce.low_u64()), call, era, $genesis_hash);
			let signature = raw_payload.using_encoded(|payload| if payload.len() > 256 {
				$from.sign(&blake2_256(payload)[..])
			} else {
				debug!("signing {}", HexDisplay::from(&payload));
				$from.sign(payload)
			});

			UncheckedExtrinsicV3 {
				signature: Some((Address::from($from.public()), signature, extra)),
				function: raw_payload.0,
			}
		}
    };
}

pub fn transfer(from: AccountKey, to: GenericAddress, amount: u128, nonce: U256, genesis_hash: Hash, node_metadata: NodeMetadata) -> BalanceExtrinsic {
	compose_extrinsic!(node_metadata, genesis_hash, BALANCES_MODULE_NAME, BALANCES_TRANSFER, nonce, from, to, Compact(amount))
}

#[cfg(test)]
mod tests {
	use node_primitives::Balance;

	use crate::Api;
	use crate::srml::system::System;
	use crypto::*;
	use codec::{Compact, Decode, Encode};
	use runtime_primitives::generic::{Era, UncheckedExtrinsic,};
	use runtime_primitives::traits::StaticLookup;
	use node_primitives::Signature;
	use crate::utils::*;
	use primitives::blake2_256;
	use keyring::AccountKeyring;
	use system as srml_system;
	use balances as srml_balances;

	use super::*;

	struct Runtime;

	impl System for Runtime {
		type Index = <node_runtime::Runtime as srml_system::Trait>::Index;
		type BlockNumber = <node_runtime::Runtime as srml_system::Trait>::BlockNumber;
		type Hash = <node_runtime::Runtime as srml_system::Trait>::Hash;
		type Hashing = <node_runtime::Runtime as srml_system::Trait>::Hashing;
		type AccountId = <node_runtime::Runtime as srml_system::Trait>::AccountId;
		type Lookup = <node_runtime::Runtime as srml_system::Trait>::Lookup;
		type Header = <node_runtime::Runtime as srml_system::Trait>::Header;
		type Event = <node_runtime::Runtime as srml_system::Trait>::Event;

		type SignedExtra = (
			srml_system::CheckGenesis<node_runtime::Runtime>,
			srml_system::CheckEra<node_runtime::Runtime>,
			srml_system::CheckNonce<node_runtime::Runtime>,
			srml_system::CheckWeight<node_runtime::Runtime>,
			srml_balances::TakeFees<node_runtime::Runtime>,
		);
		fn extra(nonce: Self::Index) -> Self::SignedExtra {
			(
				srml_system::CheckGenesis::<node_runtime::Runtime>::new(),
				srml_system::CheckEra::<node_runtime::Runtime>::from(Era::Immortal),
				srml_system::CheckNonce::<node_runtime::Runtime>::from(nonce),
				srml_system::CheckWeight::<node_runtime::Runtime>::new(),
				srml_balances::TakeFees::<node_runtime::Runtime>::from(0),
			)
		}
	}

	type Index = <Runtime as System>::Index;
	type AccountId = <Runtime as System>::AccountId;
	type Address = <<Runtime as System>::Lookup as StaticLookup>::Source;
	type TestExtrinsic = UncheckedExtrinsic<GenericAddress, BalanceTransfer, Signature, <Runtime as System>::SignedExtra>;


	#[test]
	fn call_from_meta_data_works() {
		let node_ip = "127.0.0.1";
		let node_port = "9500";
		let url = format!("{}:{}", node_ip, node_port);
		let balance_module_index = 3u8;
		let balance_transfer_index = 0u8;
		println!("Interacting with node on {}", url);

		let api = Api::new(format!("ws://{}", url));

		let amount = Balance::from(42 as u128);
		let to = AccountKey::public_from_suri("//Alice", Some(""), CryptoKind::Sr25519);

		let my_call = ([balance_module_index, balance_transfer_index], GenericAddress::from(to.clone()), Compact(amount)).encode();
		let transfer_fn = compose_call!(api.metadata.clone(), BALANCES_MODULE_NAME, BALANCES_TRANSFER, GenericAddress::from(to), Compact(amount)).encode();
		assert_eq!(my_call, transfer_fn);
	}

	#[test]
	fn custom_extrinsic_works() {
		let node_ip = "127.0.0.1";
		let node_port = "9500";
		let url = format!("{}:{}", node_ip, node_port);
		let balance_module_index = 3u8;
		let balance_transfer_index = 0u8;
		println!("Interacting with node on {}", url);

		let api = Api::new(format!("ws://{}", url));

		let accountid = AccountId::from(AccountKeyring::Alice);
		let result_str = api.get_storage("System", "AccountNonce", Some(accountid.encode())).unwrap();
		let nonce = hexstr_to_u256(result_str);
		println!("[+] Alice's Account Nonce is {}", nonce);

		let amount = Balance::from(42 as u128);
		let to = AccountKey::public_from_suri("//Alice", Some(""), CryptoKind::Sr25519);
		let from = AccountKey::new("//Alice", Some(""), CryptoKind::Sr25519);
		let hash = <Runtime as System>::Hash::from(api.genesis_hash.clone());

		let my_call = compose_call!(api.metadata.clone(), BALANCES_MODULE_NAME, BALANCES_TRANSFER, GenericAddress::from(to.clone()), Compact(amount));
		let extra = <Runtime as System>::extra(0);

		let raw_payload = (my_call.clone(), extra.clone(), (&hash, &hash.clone()));
//			let raw_payload = (Compact($nonce.low_u64()), call, era, $genesis_hash);
		let signature = raw_payload.using_encoded(|payload| if payload.len() > 256 {
			from.sign(&blake2_256(payload)[..])
		} else {
//			debug!("signing {}", HexDisplay::from(&payload));
			from.sign(payload)
		});

	 	let ux = TestExtrinsic::new_signed(
			my_call,from.public().into(), signature.into(), extra
		);

		let mut _xthex = hex::encode(ux.encode());
		_xthex.insert_str(0, "0x");


		let tx_hash = api.send_extrinsic(_xthex).unwrap();
		println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);

		println!("{:?}", ux);
		let my_ux = transfer(from, GenericAddress::from(to), amount, nonce, api.genesis_hash, api.metadata);
		println!("{:?}", my_ux);


		assert_eq!(ux.encode(), my_ux.encode());
	}
}