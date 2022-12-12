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
	api::{Api, ApiResult, GetStorage},
	rpc::Request,
	ExtrinsicParams,
};
use ac_primitives::BalancesConfig;

/// Interface to common calls of the substrate balances pallet.
pub trait GetBalance<Hash> {
	type Balance;

	fn get_existential_deposit(&self) -> ApiResult<Self::Balance>;
}

impl<Signer, Client, Params, Runtime> GetBalance<Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Runtime: BalancesConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
{
	type Balance = Runtime::Balance;

	fn get_existential_deposit(&self) -> ApiResult<Self::Balance> {
		self.get_constant("Balances", "ExistentialDeposit")
	}
}
