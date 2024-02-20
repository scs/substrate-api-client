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

/// Generates the extrinsic's call field for a given module and call passed as &str, if found in the metadata.
/// Otherwise None is returned.
/// # Arguments
///
/// * 'node_metadata' - This crate's parsed node metadata as field of the API.
/// * 'pallet_name' - Pallet name as &str for which the call is composed.
/// * 'call_name' - Call name as &str
/// * 'args' - Optional sequence of arguments of the call. They are not checked against the metadata.
#[macro_export]
macro_rules! compose_call {
($node_metadata: expr, $pallet_name: expr, $call_name: expr $(, $args: expr) *) => {
        {
			let maybe_pallet = $node_metadata.pallet_by_name($pallet_name);
			let maybe_call = match  maybe_pallet {
				Some(pallet) => {
					$crate::compose_call_for_pallet_metadata!(pallet, $call_name $(, ($args)) *)
				},
				None => None,
			};
			maybe_call
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

			let maybe_call_variant = $pallet_metadata.call_variant_by_name($call_name);
			match maybe_call_variant {
				Some(call_variant) => Some(([$pallet_metadata.index(), call_variant.index as u8] $(, ($args)) *)),
				None => None,
			}

        }

    };
}

/// Generates an UncheckedExtrinsic for a given call.
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

/// Generates an UncheckedExtrinsic for the given pallet and call, if they are found within the metadata.
/// Otherwise None is returned.
/// # Arguments
///
/// * 'api' - This instance of API. If the *signer* field is not set, an unsigned extrinsic will be generated.
/// * 'nonce' - signer's account nonce: Index
/// * 'pallet_name' - Pallet name as &str for which the call is composed.
/// * 'call_name' - Call name as &str
/// * 'args' - Optional sequence of arguments of the call. They are not checked against the metadata.
#[macro_export]
macro_rules! compose_extrinsic_with_nonce {
	($api: expr,
	$nonce: expr,
	$pallet_name: expr,
	$call_name: expr
	$(, $args: expr) *) => {
		{
            use $crate::log::debug;
            use $crate::primitives::UncheckedExtrinsicV4;

            debug!("Composing generic extrinsic for module {:?} and call {:?}", $pallet_name, $call_name);

			let metadata = $api.metadata();
            let maybe_call = $crate::compose_call!(metadata, $pallet_name, $call_name $(, ($args)) *);

			let maybe_extrinsic = match maybe_call {
				Some(call) => {
					let extrinsic = if let Some(signer) = $api.signer() {
						$crate::compose_extrinsic_offline!(
							signer,
							call.clone(),
							$api.extrinsic_params($nonce)
						)
					} else {
						UncheckedExtrinsicV4::new_unsigned(call.clone())
					};
					Some(extrinsic)
				},
				None => None,
			};
			maybe_extrinsic


		}
	};
}

/// Generates an UncheckedExtrinsic for the given pallet and call, if they are found within the metadata.
/// Otherwise None is returned.
/// Fetches the nonce from the given `api` instance. If this fails, zero is taken as default nonce.
/// See also compose_extrinsic_with_nonce
#[macro_export]
#[cfg(feature = "sync-api")]
macro_rules! compose_extrinsic {
	($api: expr,
	$pallet_name: expr,
	$call_name: expr
	$(, $args: expr) *) => {
		{
			let nonce = $api.get_nonce().unwrap_or_default();
			let maybe_extrinisc = $crate::compose_extrinsic_with_nonce!($api, nonce, $pallet_name, $call_name $(, ($args)) *);
			maybe_extrinisc
		}
    };
}

/// Generates an UncheckedExtrinsic for the given pallet and call, if they are found within the metadata.
/// Otherwise None is returned.
/// Fetches the nonce from the given `api` instance. If this fails, zero is taken as default nonce.
/// See also compose_extrinsic_with_nonce
#[macro_export]
#[cfg(not(feature = "sync-api"))]
macro_rules! compose_extrinsic {
	($api: expr,
	$pallet_name: expr,
	$call_name: expr
	$(, $args: expr) *) => {
		{
			let nonce = $api.get_nonce().await.unwrap_or_default();
			let maybe_extrinisc = $crate::compose_extrinsic_with_nonce!($api, nonce, $pallet_name, $call_name $(, ($args)) *);
			maybe_extrinisc
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
		)
		.unwrap();
		assert_eq!(expected_call_one, call_one);
		let expected_call_two = ([4, 8], extra_parameter);
		let call_two = compose_call_for_pallet_metadata!(
			&pallet_metadata,
			"force_set_balance",
			extra_parameter
		)
		.unwrap();
		assert_eq!(expected_call_two, call_two);
	}

	#[test]
	fn macro_compose_call_for_pallet_metadata_returns_none_for_unknown_function() {
		let encoded_metadata = fs::read("../ksm_metadata_v14.bin").unwrap();
		let runtime_metadata_prefixed =
			RuntimeMetadataPrefixed::decode(&mut encoded_metadata.as_slice()).unwrap();
		let metadata = Metadata::try_from(runtime_metadata_prefixed).unwrap();

		let pallet_metadata = metadata.pallet_by_name("Balances").unwrap();
		let non_existent_function = "obladi";

		let option = compose_call_for_pallet_metadata!(&pallet_metadata, non_existent_function);
		assert!(option.is_none());
	}

	#[test]
	fn macro_compose_call_returns_none_for_unknown_function() {
		let encoded_metadata = fs::read("../ksm_metadata_v14.bin").unwrap();
		let runtime_metadata_prefixed =
			RuntimeMetadataPrefixed::decode(&mut encoded_metadata.as_slice()).unwrap();
		let metadata = Metadata::try_from(runtime_metadata_prefixed).unwrap();

		let pallet_name = "Balances";
		let non_existent_function = "obladi";

		let option = compose_call!(&metadata, pallet_name, non_existent_function);
		assert!(option.is_none());
	}

	#[test]
	fn macro_compose_call_returns_none_for_unknown_pallet() {
		let encoded_metadata = fs::read("../ksm_metadata_v14.bin").unwrap();
		let runtime_metadata_prefixed =
			RuntimeMetadataPrefixed::decode(&mut encoded_metadata.as_slice()).unwrap();
		let metadata = Metadata::try_from(runtime_metadata_prefixed).unwrap();

		let pallet_name = "Balance";
		let non_existent_function = "force_set_balance";

		let option = compose_call!(&metadata, pallet_name, non_existent_function);
		assert!(option.is_none());
	}

	#[test]
	fn macro_compose_call_works_for_valid_input() {
		let encoded_metadata = fs::read("../ksm_metadata_v14.bin").unwrap();
		let runtime_metadata_prefixed =
			RuntimeMetadataPrefixed::decode(&mut encoded_metadata.as_slice()).unwrap();
		let metadata = Metadata::try_from(runtime_metadata_prefixed).unwrap();

		let pallet_name = "Balances";
		let non_existent_function = "force_set_balance";
		let extra_parameter = 10000;

		let expected_call = ([4, 8], extra_parameter);
		let call =
			compose_call!(&metadata, pallet_name, non_existent_function, extra_parameter).unwrap();
		assert_eq!(call, expected_call);
	}
}
