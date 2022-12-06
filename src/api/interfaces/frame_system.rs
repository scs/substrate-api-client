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
use sp_runtime::generic::SignedBlock;

pub type AccountInfoFor<Runtime> = frame_system::AccountInfo<
	<Runtime as frame_system::Config>::Index,
	<Runtime as frame_system::Config>::AccountData,
>;

/// Interface to common frame system pallet information.
pub trait FrameSystemInterface<Runtime: frame_system::Config> {
	type Block;

	fn get_account_info(
		&self,
		address: &Runtime::AccountId,
	) -> ApiResult<Option<AccountInfoFor<Runtime>>>;

	fn get_account_data(
		&self,
		address: &Runtime::AccountId,
	) -> ApiResult<Option<Runtime::AccountData>>;

	fn get_finalized_head(&self) -> ApiResult<Option<Runtime::Hash>>;

	fn get_header(&self, hash: Option<Runtime::Hash>) -> ApiResult<Option<Runtime::Header>>;

	fn get_block_hash(
		&self,
		number: Option<Runtime::BlockNumber>,
	) -> ApiResult<Option<Runtime::Hash>>;

	fn get_block(&self, hash: Option<Runtime::Hash>) -> ApiResult<Option<Self::Block>>;

	fn get_block_by_num(
		&self,
		number: Option<Runtime::BlockNumber>,
	) -> ApiResult<Option<Self::Block>>;

	/// A signed block is a block with Justification ,i.e., a Grandpa finality proof.
	/// The interval at which finality proofs are provided is set via the
	/// the `GrandpaConfig.justification_period` in a node's service.rs.
	/// The Justification may be None.
	fn get_signed_block(
		&self,
		hash: Option<Runtime::Hash>,
	) -> ApiResult<Option<SignedBlock<Self::Block>>>;

	fn get_signed_block_by_num(
		&self,
		number: Option<Runtime::BlockNumber>,
	) -> ApiResult<Option<SignedBlock<Self::Block>>>;
}
