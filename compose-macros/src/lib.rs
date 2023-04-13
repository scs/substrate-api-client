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

#![cfg_attr(not(feature = "std"), no_std)]

// re-export for macro resolution
pub use ac_primitives as primitives;
pub use log;

mod rpc;

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
($node_metadata: expr, $pallet: expr, $call_name: expr $(, $args: expr) *) => {
        {
            let pallet = $node_metadata.pallet($pallet).unwrap().to_owned();

            let call_index = pallet.call_indexes.get($call_name).unwrap();

            ([pallet.index, *call_index as u8] $(, ($args)) *)
        }
    };
}

/// Generates an Unchecked extrinsic for a given call
/// # Arguments
///
/// * 'signer' - AccountKey that is used to sign the extrinsic.
/// * 'call' - call as returned by the compose_call! macro or via substrate's call enums.
/// * 'nonce' - signer's account nonce: u32
/// * 'era' - Era for extrinsic to be valid
/// * 'genesis_hash' - sp-runtime::Hash256/[u8; 32].
/// * 'runtime_spec_version' - RuntimeVersion.spec_version/u32
#[macro_export]
macro_rules! compose_extrinsic_offline {
	($signer: expr,
    $call: expr,
    $params: expr) => {{
		use $crate::primitives::extrinsics::{
			ExtrinsicParams, SignExtrinsic, SignedPayload, UncheckedExtrinsicV4,
		};

		let extra = $params.signed_extra();
		let raw_payload =
			SignedPayload::from_raw($call.clone(), extra, $params.additional_signed());

		let signature = raw_payload.using_encoded(|payload| $signer.sign(payload));

		UncheckedExtrinsicV4::new_signed($call, $signer.extrinsic_address(), signature, extra)
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
macro_rules! compose_extrinsic {
	($api: expr,
	$module: expr,
	$call: expr
	$(, $args: expr) *) => {
		{
            use $crate::log::debug;
            use $crate::primitives::UncheckedExtrinsicV4;

            debug!("Composing generic extrinsic for module {:?} and call {:?}", $module, $call);
            let call = $crate::compose_call!($api.metadata().clone(), $module, $call $(, ($args)) *);
            if let Some(signer) = $api.signer() {
                $crate::compose_extrinsic_offline!(
                    signer,
                    call.clone(),
                    $api.extrinsic_params($api.get_nonce().unwrap())
                )
            } else {
                UncheckedExtrinsicV4::new_unsigned(call.clone())
            }
		}
    };
}
