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

use crate::api::ApiResult;
use codec::Decode;
use frame_metadata::RuntimeMetadataPrefixed;
use sp_version::RuntimeVersion;

/// Interface to runtime specified variables.
pub trait RuntimeInterface {
	fn get_metadata(&self) -> ApiResult<RuntimeMetadataPrefixed>;

	fn get_runtime_version(&self) -> ApiResult<RuntimeVersion>;

	fn get_constant<C: Decode>(&self, pallet: &'static str, constant: &'static str)
		-> ApiResult<C>;
}
