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

//! Offers some predefined extrinsics for common runtime modules.

pub use balances::CreateBalancesExtrinsic;

pub mod balances;
pub mod common;
pub mod contracts;
pub mod offline_extrinsic;
#[cfg(feature = "staking-xt")]
pub mod staking;
pub mod utility;

use crate::Api;
use ac_primitives::{ExtrinsicParams, FrameSystemConfig, SignExtrinsic, UncheckedExtrinsicV4};

pub type AddressFor<Module> = <Module as AssignExtrinsicTypes>::Address;
pub type SignatureFor<Module> = <Module as AssignExtrinsicTypes>::Signature;
pub type SignedExtraFor<Module> = <Module as AssignExtrinsicTypes>::SignedExtra;

type ExtrinsicFor<Module, Call> =
	UncheckedExtrinsicV4<AddressFor<Module>, Call, SignatureFor<Module>, SignedExtraFor<Module>>;

pub trait AssignExtrinsicTypes {
	type Address;
	type Signature;
	type SignedExtra;
}

impl<Signer, Client, Params, Runtime> AssignExtrinsicTypes for Api<Signer, Client, Params, Runtime>
where
	Signer: SignExtrinsic<Runtime::AccountId>,
	Runtime: FrameSystemConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
{
	type Address = Signer::ExtrinsicAddress;
	type Signature = Signer::Signature;
	type SignedExtra = Params::SignedExtra;
}
