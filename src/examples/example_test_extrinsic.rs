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

// NOTE: Used for debugging purposes. TO BE REMOVED

use balances as srml_balances;
use codec::{Compact, Encode};
use keyring::AccountKeyring;
use node_primitives::Balance;
use node_primitives::Signature;
use primitives::blake2_256;
use runtime_primitives::generic::{Era, UncheckedExtrinsic};
//use runtime_primitives::traits::StaticLookup;
use system as srml_system;

use substrate_api_client::Api;
use substrate_api_client::compose_call;
use substrate_api_client::extrinsic::crypto::*;
use substrate_api_client::extrinsic::definitions::*;
use substrate_api_client::srml::system::System;
use substrate_api_client::utils::*;

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

// use our own generic data types instead of these
//type Index = <Runtime as System>::Index;
//type Address = <<Runtime as System>::Lookup as StaticLookup>::Source;
type AccountId = <Runtime as System>::AccountId;
type TestExtrinsic = UncheckedExtrinsic<GenericAddress, BalanceTransfer, Signature, <Runtime as System>::SignedExtra>;

fn main() {
    let node_ip = "127.0.0.1";
    let node_port = "9991";
    let url = format!("{}:{}", node_ip, node_port);
    println!("Interacting with node on {}", url);

    let api = Api::new(format!("ws://{}", url));

    let accountid = AccountId::from(AccountKeyring::Alice);
    let result_str = api.get_storage("System", "AccountNonce", Some(accountid.encode())).unwrap();
    let nonce = hexstr_to_u256(result_str);
    println!("[+] Alice's Account Nonce is {}", nonce);

    let amount = Balance::from(42 as u128);
    let from = AccountKey::new("//Alice", Some(""), CryptoKind::Sr25519);
    let to = AccountKey::public_from_suri("//Bob", Some(""), CryptoKind::Sr25519);
    let hash = <Runtime as System>::Hash::from(api.genesis_hash.clone());

    let my_call = compose_call!(api.metadata.clone(), BALANCES_MODULE_NAME, BALANCES_TRANSFER, GenericAddress::from(to.clone()), Compact(amount));
    let extra = <Runtime as System>::extra(nonce.low_u32());

    let raw_payload = (my_call, extra.clone(), (&hash, &hash));
    let signature = raw_payload.using_encoded(|payload| if payload.len() > 256 {
        from.sign(&blake2_256(payload)[..])
    } else {
//        println!("signing {}", HexDisplay::from(&payload));
        from.sign(payload)
    });

    let ux = TestExtrinsic::new_signed(
        raw_payload.0, GenericAddress::from(from.public()), signature.into(), extra
    );

    let mut _xthex = hex::encode(ux.encode());
    _xthex.insert_str(0, "0x");

    let tx_hash = api.send_extrinsic(_xthex).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);
}