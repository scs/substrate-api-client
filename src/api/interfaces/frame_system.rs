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

use crate::{api::ApiResult, rpc::json_req, Api, RpcClient};
use ac_primitives::{AccountInfo, ExtrinsicParams, FrameSystemConfig};
use log::*;
use sp_runtime::generic::SignedBlock;

pub type AccountInfoFor<T> =
	AccountInfo<<T as FrameSystemConfig>::Index, <T as FrameSystemConfig>::AccountData>;

/// Interface to common frame system pallet information.
pub trait GetFrameSystemInterface<Runtime: FrameSystemConfig> {
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

impl<Signer, Client, Params, Runtime> GetFrameSystemInterface<Runtime>
	for Api<Signer, Client, Params, Runtime>
where
	Client: RpcClient,
	Runtime: FrameSystemConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
{
	type Block = Runtime::Block;

	fn get_account_info(
		&self,
		address: &Runtime::AccountId,
	) -> ApiResult<Option<AccountInfoFor<Runtime>>> {
		let storagekey: sp_core::storage::StorageKey = self
			.metadata
			.storage_map_key::<Runtime::AccountId>("System", "Account", address.clone())?;

		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key_hash(storagekey, None)
	}

	fn get_account_data(
		&self,
		address: &Runtime::AccountId,
	) -> ApiResult<Option<Runtime::AccountData>> {
		self.get_account_info(address).map(|info| info.map(|i| i.data))
	}

	fn get_finalized_head(&self) -> ApiResult<Option<Runtime::Hash>> {
		let h = self.get_request(json_req::chain_get_finalized_head())?;
		match h {
			Some(hash) => Ok(Some(Runtime::Hash::from_hex(hash)?)),
			None => Ok(None),
		}
	}

	fn get_header(&self, hash: Option<Runtime::Hash>) -> ApiResult<Option<Runtime::Header>> {
		let h = self.get_request(json_req::chain_get_header(hash))?;
		match h {
			Some(hash) => Ok(Some(serde_json::from_str(&hash)?)),
			None => Ok(None),
		}
	}

	fn get_block_hash(
		&self,
		number: Option<Runtime::BlockNumber>,
	) -> ApiResult<Option<Runtime::Hash>> {
		let h = self.get_request(json_req::chain_get_block_hash(number))?;
		match h {
			Some(hash) => Ok(Some(Runtime::Hash::from_hex(hash)?)),
			None => Ok(None),
		}
	}

	fn get_block(&self, hash: Option<Runtime::Hash>) -> ApiResult<Option<Self::Block>> {
		Self::get_signed_block(self, hash).map(|sb_opt| sb_opt.map(|sb| sb.block))
	}

	fn get_block_by_num(
		&self,
		number: Option<Runtime::BlockNumber>,
	) -> ApiResult<Option<Self::Block>> {
		Self::get_signed_block_by_num(self, number).map(|sb_opt| sb_opt.map(|sb| sb.block))
	}

	fn get_signed_block(
		&self,
		hash: Option<Runtime::Hash>,
	) -> ApiResult<Option<SignedBlock<Self::Block>>> {
		let b = self.get_request(json_req::chain_get_block(hash))?;
		match b {
			Some(block) => Ok(Some(serde_json::from_str(&block)?)),
			None => Ok(None),
		}
	}

	fn get_signed_block_by_num(
		&self,
		number: Option<Runtime::BlockNumber>,
	) -> ApiResult<Option<SignedBlock<Self::Block>>> {
		self.get_block_hash(number).map(|h| self.get_signed_block(h))?
	}
}
