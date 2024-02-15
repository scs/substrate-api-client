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
/// * 'pallet' - Pallet name as &str for which the call is composed.
/// * 'call_name' - Call name as &str
/// * 'args' - Optional sequence of arguments of the call. They are not checked against the metadata.
/// As of now the user needs to check himself that the correct arguments are supplied.
#[macro_export]
macro_rules! compose_call {
($node_metadata: expr, $pallet: expr, $call_name: expr $(, $args: expr) *) => {
        {
			let pallet_metadata = $node_metadata.pallet_by_name($pallet).unwrap().to_owned();
            $crate::compose_call_for_pallet_metadata!(pallet_metadata, $call_name $(, ($args)) *)
        }
    };
}

/// Generates the extrinsic's call field for the given PalletMetadata
/// # Arguments
///
/// * 'pallet_metadata' - This crate's parsed pallet metadata as field of the API.
/// * 'call_name' - Call name as &str
/// * 'args' - Optional sequence of arguments of the call. They are not checked against the metadata.
/// As of now the user needs to check himself that the correct arguments are supplied.
#[macro_export]
macro_rules! compose_call_for_pallet_metadata {
($pallet_metadata: expr, $call_name: expr $(, $args: expr) *) => {
        {
            let call_index = $pallet_metadata.call_variant_by_name($call_name).unwrap().index;
            ([$pallet_metadata.index(), call_index as u8] $(, ($args)) *)
        }
    };
}

/// Generates an Unchecked extrinsic for a given call
/// # Arguments
///
/// * 'signer' - AccountKey that is used to sign the extrinsic.
/// * 'call' - call as returned by the compose_call! macro or via substrate's call enums.
/// * 'params' - Instance of `ExtrinsicParams` that can be used to fetch signed extra and additional signed
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
/// * 'nonce' - signer's account nonce: Index
/// * 'module' - Module name as &str for which the call is composed.
/// * 'call' - Call name as &str
/// * 'args' - Optional sequence of arguments of the call. They are not checked against the metadata.
/// As of now the user needs to check himself that the correct arguments are supplied.
#[macro_export]
macro_rules! compose_extrinsic_with_nonce {
	($api: expr,
	$nonce: expr,
	$module: expr,
	$call: expr
	$(, $args: expr) *) => {
		{
            use $crate::log::debug;
            use $crate::primitives::UncheckedExtrinsicV4;

            debug!("Composing generic extrinsic for module {:?} and call {:?}", $module, $call);

			let metadata = $api.metadata();
            let call = $crate::compose_call!(metadata, $module, $call $(, ($args)) *);
            if let Some(signer) = $api.signer() {
                $crate::compose_extrinsic_offline!(
                    signer,
                    call.clone(),
                    $api.extrinsic_params($nonce)
                )
            } else {
                UncheckedExtrinsicV4::new_unsigned(call.clone())
            }
		}
	};
}

/// Generates an Unchecked extrinsic for a given module and call passed as a &str.
/// Fetches the nonce from the given `api` instance
/// See also compose_extrinsic_with_nonce
#[macro_export]
#[cfg(feature = "sync-api")]
macro_rules! compose_extrinsic {
	($api: expr,
	$module: expr,
	$call: expr
	$(, $args: expr) *) => {
		{
			let nonce = $api.get_nonce().unwrap();
			let extrinsic = $crate::compose_extrinsic_with_nonce!($api, nonce, $module, $call $(, ($args)) *);
			extrinsic
		}
    };
}

/// Generates an Unchecked extrinsic for a given module and call passed as a &str.
/// Fetches the nonce from the given `api` instance
/// See also compose_extrinsic_with_nonce
#[macro_export]
#[cfg(not(feature = "sync-api"))]
macro_rules! compose_extrinsic {
	($api: expr,
	$module: expr,
	$call: expr
	$(, $args: expr) *) => {
		{
			let nonce = $api.get_nonce().await.unwrap();
			let extrinsic = $crate::compose_extrinsic_with_nonce!($api, nonce, $module, $call $(, ($args)) *);
			extrinsic
		}
	};
}

#[cfg(test)]
mod tests {
	use super::*;
	use ac_node_api::Metadata;
	use codec::Decode;
	use frame_metadata::RuntimeMetadataPrefixed;
	use std::fs;

	#[test]
	fn macro_compose_call_for_pallet_metadata_works() {
		let encoded_metadata = fs::read("../ksm_metadata_v14.bin").unwrap();
		let runtime_metadata_prefixed =
			RuntimeMetadataPrefixed::decode(&mut encoded_metadata.as_slice()).unwrap();
		let metadata = Metadata::try_from(runtime_metadata_prefixed).unwrap();

		let pallet_metadata = metadata.pallet_by_name("Balances").unwrap();

		let extra_parameter = 10000;
		let expected_call_one = ([4, 0], extra_parameter);
		let call_one = compose_call_for_pallet_metadata!(
			&pallet_metadata,
			"transfer_allow_death",
			extra_parameter
		);
		assert_eq!(expected_call_one, call_one);
		let expected_call_two = ([4, 8], extra_parameter);
		let call_two = compose_call_for_pallet_metadata!(
			&pallet_metadata,
			"force_set_balance",
			extra_parameter
		);
		assert_eq!(expected_call_two, call_two);
	}
}
