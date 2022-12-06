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
use crate::{
	api::{Api, ApiClientError, ApiResult},
	rpc::{json_req, RpcClient},
	ExtrinsicParams,
};
pub use pallet_transaction_payment::FeeDetails;
use pallet_transaction_payment::{InclusionFee, RuntimeDispatchInfo};
use sp_rpc::number::NumberOrHex;

/// Interface to common calls of the substrate balances pallet.
pub trait BalanceInterface<Runtime: pallet_balances::Config + frame_system::Config> {
	fn get_fee_details(
		&self,
		xthex_prefixed: &str,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<FeeDetails<Runtime::Balance>>>;

	fn get_payment_info(
		&self,
		xthex_prefixed: &str,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<RuntimeDispatchInfo<Runtime::Balance>>>;

	fn get_existential_deposit(&self) -> ApiResult<Runtime::Balance>;
}

// impl<Signer, Client, Params, Runtime> BalanceInterface<Runtime>
// 	for Api<Signer, Client, Params, Runtime>
// where
// 	Client: RpcClient,
// 	Runtime: pallet_balances::Config + frame_system::Config,
// 	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
// {
// 	fn get_fee_details(
// 		&self,
// 		xthex_prefixed: &str,
// 		at_block: Option<Runtime::Hash>,
// 	) -> ApiResult<Option<FeeDetails<Runtime::Balance>>> {
// 		let jsonreq = json_req::payment_query_fee_details(xthex_prefixed, at_block);
// 		let res = self.get_request(jsonreq)?;
// 		match res {
// 			Some(details) => {
// 				let details: FeeDetails<NumberOrHex> = serde_json::from_str(&details)?;
// 				let details = convert_fee_details(details)?;
// 				Ok(Some(details))
// 			},
// 			None => Ok(None),
// 		}
// 	}
//
// 	fn get_payment_info(
// 		&self,
// 		xthex_prefixed: &str,
// 		at_block: Option<Runtime::Hash>,
// 	) -> ApiResult<Option<RuntimeDispatchInfo<Runtime::Balance>>> {
// 		let jsonreq = json_req::payment_query_info(xthex_prefixed, at_block);
// 		let res = self.get_request(jsonreq)?;
// 		match res {
// 			Some(info) => {
// 				let info: RuntimeDispatchInfo<Runtime::Balance> = serde_json::from_str(&info)?;
// 				Ok(Some(info))
// 			},
// 			None => Ok(None),
// 		}
// 	}
//
// 	fn get_existential_deposit(&self) -> ApiResult<Runtime::Balance> {
// 		self.get_constant("Balances", "ExistentialDeposit")
// 	}
// }
//
// fn convert_fee_details<Balance>(
// 	details: FeeDetails<NumberOrHex>,
// ) -> ApiResult<FeeDetails<Balance>> {
// 	let inclusion_fee = if let Some(inclusion_fee) = details.inclusion_fee {
// 		Some(inclusion_fee_with_balance(inclusion_fee)?)
// 	} else {
// 		None
// 	};
// 	let tip = details.tip.try_into().map_err(|_| ApiClientError::TryFromIntError)?;
// 	Ok(FeeDetails { inclusion_fee, tip })
// }
//
// fn inclusion_fee_with_balance<Balance>(
// 	inclusion_fee: InclusionFee<NumberOrHex>,
// ) -> ApiResult<InclusionFee<Balance>> {
// 	Ok(InclusionFee {
// 		base_fee: inclusion_fee.base_fee.try_into().map_err(|_| ApiClientError::TryFromIntError)?,
// 		len_fee: inclusion_fee.len_fee.try_into().map_err(|_| ApiClientError::TryFromIntError)?,
// 		adjusted_weight_fee: inclusion_fee
// 			.adjusted_weight_fee
// 			.try_into()
// 			.map_err(|_| ApiClientError::TryFromIntError)?,
// 	})
// }
