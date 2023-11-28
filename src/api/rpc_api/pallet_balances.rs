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
	api::{Api, GetStorage, Result},
	rpc::Request,
};
use ac_primitives::config::Config;
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;

/// Interface to common calls of the substrate balances pallet.
#[maybe_async::maybe_async(?Send)]
pub trait GetBalance {
	type Balance;

	async fn get_existential_deposit(&self) -> Result<Self::Balance>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> GetBalance for Api<T, Client>
where
	T: Config,
	Client: Request,
{
	type Balance = T::Balance;

	async fn get_existential_deposit(&self) -> Result<Self::Balance> {
		self.get_constant("Balances", "ExistentialDeposit").await
	}
}
