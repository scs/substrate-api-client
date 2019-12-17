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

//! Offers macros that build extrinsics for custom runtime modules based on the metadata.
//! Additionally, some predefined extrinsics for common runtime modules are implemented.

#[cfg(feature = "std")]
pub extern crate codec;
#[cfg(feature = "std")]
pub extern crate log;
pub extern crate node_primitives;

pub mod balances;
pub mod contract;
pub mod xt_primitives;

/// Generates the extrinsic's call field for a given module and call passed as &str
/// # Arguments
///
/// * 'node_metadata' - This crate's parsed node metadata as field of the API.
/// * 'module' - Module name as &str for which the call is composed.
/// * 'call' - Call name as &str
/// * 'args' - Optional sequence of arguments of the call. They are not checked against the metadata.
/// As of now the user needs to check himself that the correct arguments are supplied.
#[macro_export]
macro_rules! compose_call {
($node_metadata: expr, $module: expr, $call_name: expr $(, $args: expr) *) => {
        {
            let mut meta = $node_metadata;
            meta.retain(|m| !m.calls.is_empty());

            let module_index = meta
            .iter().position(|m| m.name == $module).expect("Module not found in Metadata");

            let call_index = meta[module_index].calls
            .iter().position(|c| c.name == $call_name).expect("Call not found in Module");

            ([module_index as u8, call_index as u8] $(, ($args)) *)
        }
    };
}

/// Generates an Unchecked extrinsic for a given call
/// # Arguments
///
/// * 'signer' - AccountKey that is used to sign the extrinsic.
/// * 'call' - call as returned by the compose_call! macro or via substrate's call enums.
/// * 'nonce' - signer's account nonce: u32
/// * 'genesis_hash' - sr-primitives::Hash256/[u8; 32].
/// * 'runtime_spec_version' - RuntimeVersion.spec_version/u32
#[macro_export]
macro_rules! compose_extrinsic_offline {
    ($signer: expr,
    $call: expr,
    $nonce: expr,
    $genesis_hash: expr,
    $runtime_spec_version: expr) => {{
        use $crate::extrinsic::xt_primitives::*;
        use $crate::extrinsic::node_primitives::AccountId;

        let extra = GenericExtra::new($nonce);
        let raw_payload = SignedPayload::from_raw(
            $call.clone(),
            extra.clone(),
            (
                $runtime_spec_version,
                $genesis_hash,
                $genesis_hash,
                (),
                (),
                (),
                (),
            ),
        );

        let signature = raw_payload.using_encoded(|payload| $signer.sign(payload));

        let mut arr: [u8; 32] = Default::default();
        arr.clone_from_slice($signer.public().as_ref());

        UncheckedExtrinsicV4::new_signed(
            $call,
            GenericAddress::from(AccountId::from(arr)),
            signature.into(),
            extra
        )
    }};
}

/// Generates an Unchecked extrinsic for a given module and call passed as a &str.
/// # Arguments
///
/// * 'api' - This instance of API. If the *signer* field is not set, an unsigned extrinsic will be generated.
/// * 'module' - Module name as &str for which the call is composed.
/// * 'call' - Call name as &str
/// * 'args' - Optional sequence of arguments of the call. They are not checked against the metadata.
/// As of now the user needs to check himself that the correct arguments are supplied.

#[macro_export]
#[cfg(feature = "std")]
macro_rules! compose_extrinsic {
	($api: expr,
	$module: expr,
	$call: expr
	$(, $args: expr) *) => {
		{
            use $crate::extrinsic::codec::Compact;
            use $crate::extrinsic::log::info;
            use $crate::extrinsic::xt_primitives::*;

            info!("Composing generic extrinsic for module {:?} and call {:?}", $module, $call);
            let call = $crate::compose_call!($api.metadata.clone(), $module, $call $(, ($args)) *);

            if let Some(signer) = $api.signer.clone() {
                $crate::compose_extrinsic_offline!(
                    signer,
                    call.clone(),
                    $api.get_nonce().await.unwrap(),
                    $api.genesis_hash,
                    $api.runtime_version.spec_version
                )
            } else {
                UncheckedExtrinsicV4 {
                    signature: None,
                    function: call.clone(),
                }
            }
		}
    };
}
