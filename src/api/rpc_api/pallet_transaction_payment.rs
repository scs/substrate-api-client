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
use ac_primitives::{config::Config, FeeDetails, InclusionFee, NumberOrHex, RuntimeDispatchInfo};
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use core::str::FromStr;
use sp_core::Bytes;
/// Interface to common calls of the substrate transaction payment pallet.
#[maybe_async::maybe_async(?Send)]
pub trait GetTransactionPayment {
	type Hash;
	type Balance;

	async fn get_fee_details(
		&self,
		encoded_extrinsic: &Bytes,
		at_block: Option<Self::Hash>,
	) -> Result<Option<FeeDetails<Self::Balance>>>;

	async fn get_payment_info(
		&self,
		encoded_extrinsic: &Bytes,
		at_block: Option<Self::Hash>,
	) -> Result<Option<RuntimeDispatchInfo<Self::Balance>>>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> GetTransactionPayment for Api<T, Client>
where
	T: Config,
	Client: Request,
	T::Balance: TryFrom<NumberOrHex> + FromStr,
{
	type Hash = T::Hash;
	type Balance = T::Balance;

	async fn get_fee_details(
		&self,
		encoded_extrinsic: &Bytes,
		at_block: Option<Self::Hash>,
	) -> Result<Option<FeeDetails<Self::Balance>>> {
		let details: Option<FeeDetails<NumberOrHex>> = self
			.client()
			.request("payment_queryFeeDetails", rpc_params![encoded_extrinsic, at_block])
			.await?;

		let details = match details {
			Some(details) => Some(convert_fee_details(details)?),
			None => None,
		};
		Ok(details)
	}

	async fn get_payment_info(
		&self,
		encoded_extrinsic: &Bytes,
		at_block: Option<Self::Hash>,
	) -> Result<Option<RuntimeDispatchInfo<Self::Balance>>> {
		let res = self
			.client()
			.request("payment_queryInfo", rpc_params![encoded_extrinsic, at_block])
			.await?;
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
