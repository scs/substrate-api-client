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

use crypto::AccountKey;
use xt_primitives::*;

use crate::node_metadata::NodeMetadata;
use crate::crypto;

pub mod xt_primitives;

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

pub fn transfer(from: AccountKey, to: GenericAddress, amount: u128, nonce: U256, genesis_hash: Hash, node_metadata: NodeMetadata) -> BalanceTransferXt {
	compose_extrinsic!(
		node_metadata,
		genesis_hash,
		BALANCES_MODULE,
		BALANCES_TRANSFER,
		GenericExtra::new(nonce.low_u32()),
		from,
		to,
		Compact(amount)
	)
}

pub fn contract_put_code(from: AccountKey, gas_limit: u64, code: Vec<u8>, nonce: U256, genesis_hash: Hash, node_metadata: NodeMetadata) -> ContractPutCodeXt {
	compose_extrinsic!(
		node_metadata,
		genesis_hash,
		CONTRACTS_MODULE,
		CONTRACTS_PUT_CODE,
		GenericExtra::new(nonce.low_u32()),
		from,
		Compact(gas_limit),
		code
	)
}

pub fn contract_create(from: AccountKey, endowment: u128, gas_limit: u64, code_hash: Hash, data: Vec<u8>, nonce: U256, genesis_hash: Hash, node_metadata: NodeMetadata) -> ContractCreateXt {
    compose_extrinsic!(
		node_metadata,
		genesis_hash,
		CONTRACTS_MODULE,
		CONTRACTS_CREATE,
		GenericExtra::new(nonce.low_u32()),
		from,
		Compact(endowment),
		Compact(gas_limit),
		code_hash,
		data
	)
}

pub fn contract_call(from: AccountKey, dest: GenericAddress, value: u128, gas_limit: u64, data: Vec<u8>, nonce: U256, genesis_hash: Hash, node_metadata: NodeMetadata) -> ContractCallXt {
	compose_extrinsic!(
		node_metadata,
		genesis_hash,
		CONTRACTS_MODULE,
		CONTRACTS_CALL,
		GenericExtra::new(nonce.low_u32()),
		from,
		dest,
		Compact(value),
		Compact(gas_limit),
		data
	)
}

#[cfg(test)]
mod tests {
	use balances as srml_balances;
	use codec::{Compact, Encode};
	use keyring::AccountKeyring;
	use node_primitives::Balance;
	use node_primitives::Signature;
	use primitives::{blake2_256, hexdisplay::HexDisplay};
	use runtime_primitives::generic::{Era, UncheckedExtrinsic,};
	use runtime_primitives::traits::StaticLookup;
	use system as srml_system;

	use crypto::*;

	use crate::Api;
	use crate::srml::system::System;
	use crate::utils::*;

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

//	type Index = <Runtime as System>::Index;
	type AccountId = <Runtime as System>::AccountId;
//	type Address = <<Runtime as System>::Lookup as StaticLookup>::Source;
	type TestExtrinsic = UncheckedExtrinsic<GenericAddress, BalanceTransferFn, Signature, <Runtime as System>::SignedExtra>;

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

	#[test]
	fn custom_extrinsic_works() {
		let api = test_api();

		let accountid = AccountId::from(AccountKeyring::Alice);
		let result_str = api.get_storage("System", "AccountNonce", Some(accountid.encode())).unwrap();
		let nonce = hexstr_to_u256(result_str);
		println!("[+] Alice's Account Nonce is {}", nonce);

		let amount = Balance::from(42 as u128);
		let from = AccountKey::new("//Alice", Some(""), CryptoKind::Sr25519);
		let to = AccountKey::public_from_suri("//Bob", Some(""), CryptoKind::Sr25519);
		let hash = <Runtime as System>::Hash::from(api.genesis_hash.clone());

		let extra = <Runtime as System>::extra(nonce.low_u32());
		let gen_extra = GenericExtra::new(nonce.low_u32());

		assert_eq!(extra.encode(), gen_extra.encode());

		let ux = compose_extrinsic!(
			api.metadata.clone(),
			api.genesis_hash.clone(),
			BALANCES_MODULE,
			BALANCES_TRANSFER,
			gen_extra.clone(),
			from,
			GenericAddress::from(to),
			Compact(amount)
		);

		let mut _xthex = hex::encode(ux.encode());
		_xthex.insert_str(0, "0x");

		let tx_hash = api.send_extrinsic(_xthex).unwrap();
		println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);
	}

	#[test]
	fn tests_contract_put_code() {
		let api = test_api();

		let from = AccountKey::new("//Alice", Some(""), CryptoKind::Sr25519);
		let accountid = AccountId::from(AccountKeyring::Alice);
		let result_str = api.get_storage("System", "AccountNonce", Some(accountid.encode())).unwrap();
		let nonce = hexstr_to_u256(result_str);
		println!("[+] Alice's Account Nonce is {}", nonce);


		const CONTRACT: &str = r#"
(module
    (func (export "call"))
    (func (export "deploy"))
)
"#;
		let wasm = wabt::wat2wasm(CONTRACT).expect("invalid wabt");
		let xt = contract_put_code(from, 500_000, wasm, nonce, api.genesis_hash.clone(), api.metadata.clone());

		let mut _xthex = hex::encode(xt.encode());
		_xthex.insert_str(0, "0x");

		let tx_hash = api.send_extrinsic(_xthex).unwrap();
		println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);
	}
}