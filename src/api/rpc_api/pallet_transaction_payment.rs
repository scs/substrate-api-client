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
	api::{Api, Error, Result},
	rpc::Request,
};
use ac_compose_macros::rpc_params;
use ac_primitives::{
	BalancesConfig, Bytes, ExtrinsicParams, FeeDetails, InclusionFee, NumberOrHex,
	RuntimeDispatchInfo,
};
use core::str::FromStr;

/// Interface to common calls of the substrate transaction payment pallet.
pub trait GetTransactionPayment {
	type Hash;
	type Balance;

	fn get_fee_details(
		&self,
		encoded_extrinsic: Bytes,
		at_block: Option<Self::Hash>,
	) -> Result<Option<FeeDetails<Self::Balance>>>;

	fn get_payment_info(
		&self,
		encoded_extrinsic: Bytes,
		at_block: Option<Self::Hash>,
	) -> Result<Option<RuntimeDispatchInfo<Self::Balance>>>;
}

impl<Signer, Client, Params, Runtime> GetTransactionPayment for Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Runtime: BalancesConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime::Balance: TryFrom<NumberOrHex> + FromStr,
{
	type Hash = Runtime::Hash;
	type Balance = Runtime::Balance;

	fn get_fee_details(
		&self,
		encoded_extrinsic: Bytes,
		at_block: Option<Self::Hash>,
	) -> Result<Option<FeeDetails<Self::Balance>>> {
		let details: Option<FeeDetails<NumberOrHex>> = self
			.client()
			.request("payment_queryFeeDetails", rpc_params![encoded_extrinsic, at_block])?;

		let details = match details {
			Some(details) => Some(convert_fee_details(details)?),
			None => None,
		};
		Ok(details)
	}

	fn get_payment_info(
		&self,
		encoded_extrinsic: Bytes,
		at_block: Option<Self::Hash>,
	) -> Result<Option<RuntimeDispatchInfo<Self::Balance>>> {
		let res = self
			.client()
			.request("payment_queryInfo", rpc_params![encoded_extrinsic, at_block])?;
		Ok(res)
	}
}

fn convert_fee_details<Balance: TryFrom<NumberOrHex>>(
	details: FeeDetails<NumberOrHex>,
) -> Result<FeeDetails<Balance>> {
	let inclusion_fee = if let Some(inclusion_fee) = details.inclusion_fee {
		Some(inclusion_fee_with_balance(inclusion_fee)?)
	} else {
		None
	};
	let tip = details.tip.try_into().map_err(|_| Error::TryFromIntError)?;
	Ok(FeeDetails { inclusion_fee, tip })
}

fn inclusion_fee_with_balance<Balance: TryFrom<NumberOrHex>>(
	inclusion_fee: InclusionFee<NumberOrHex>,
) -> Result<InclusionFee<Balance>> {
	Ok(InclusionFee {
		base_fee: inclusion_fee.base_fee.try_into().map_err(|_| Error::TryFromIntError)?,
		len_fee: inclusion_fee.len_fee.try_into().map_err(|_| Error::TryFromIntError)?,
		adjusted_weight_fee: inclusion_fee
			.adjusted_weight_fee
			.try_into()
			.map_err(|_| Error::TryFromIntError)?,
	})
}
