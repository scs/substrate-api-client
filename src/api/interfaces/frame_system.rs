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

//! Interface to common frame system pallet information.

use crate::{
	api::{interfaces::generic_storage::GetGenericStorage, ApiResult},
	rpc::json_req,
	Api, FromHexString, RpcClient,
};
use ac_primitives::{AccountInfo, ExtrinsicParams, FrameSystemConfig};
use log::*;
use serde::de::DeserializeOwned;
use sp_core::{storage::StorageKey, Pair};
use sp_runtime::{generic::SignedBlock, traits::GetRuntimeBlockType, MultiSignature};

pub trait GetAccountInformation<AccountId> {
	type Index;
	type AccountData;

	fn get_account_info(
		&self,
		address: &AccountId,
	) -> ApiResult<Option<AccountInfo<Self::Index, Self::AccountData>>>;

	fn get_account_data(&self, address: &AccountId) -> ApiResult<Option<Self::AccountData>>;
}

pub trait GetHeader<Hash> {
	type Header;

	fn get_finalized_head(&self) -> ApiResult<Option<Hash>>;

	fn get_header(&self, hash: Option<Hash>) -> ApiResult<Option<Self::Header>>;
}

pub trait GetBlock<Number, Hash> {
	type Block;

	fn get_block_hash(&self, number: Option<Number>) -> ApiResult<Option<Hash>>;

	fn get_block(&self, hash: Option<Hash>) -> ApiResult<Option<Self::Block>>;

	fn get_block_by_num(&self, number: Option<Number>) -> ApiResult<Option<Self::Block>>;

	/// A signed block is a block with Justification ,i.e., a Grandpa finality proof.
	/// The interval at which finality proofs are provided is set via the
	/// the `GrandpaConfig.justification_period` in a node's service.rs.
	/// The Justification may be None.
	fn get_signed_block(&self, hash: Option<Hash>) -> ApiResult<Option<SignedBlock<Self::Block>>>;

	fn get_signed_block_by_num(
		&self,
		number: Option<Number>,
	) -> ApiResult<Option<SignedBlock<Self::Block>>>;
}

impl<Signer, Client, Params, Runtime> GetAccountInformation<Runtime::AccountId>
	for Api<Signer, Client, Params, Runtime>
where
	Signer: Pair,
	MultiSignature: From<Signer::Signature>,
	Client: RpcClient,
	Runtime: FrameSystemConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
{
	type Index = Runtime::Index;
	type AccountData = Runtime::AccountData;

	fn get_account_info(
		&self,
		address: &Runtime::AccountId,
	) -> ApiResult<Option<AccountInfo<Self::Index, Self::AccountData>>> {
		let storagekey: StorageKey = self.metadata().storage_map_key::<Runtime::AccountId>(
			"System",
			"Account",
			address.clone(),
		)?;

		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key_hash(storagekey, None)
	}

	fn get_account_data(
		&self,
		address: &Runtime::AccountId,
	) -> ApiResult<Option<Runtime::AccountData>> {
		self.get_account_info(address).map(|info| info.map(|i| i.data))
	}
}

impl<Signer, Client, Params, Runtime> GetHeader<Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Signer: Pair,
	MultiSignature: From<Signer::Signature>,
	Client: RpcClient,
	Runtime: FrameSystemConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime::Header: DeserializeOwned,
	Runtime::Hash: FromHexString,
{
	type Header = Runtime::Header;

	fn get_finalized_head(&self) -> ApiResult<Option<Runtime::Hash>> {
		let h = self.client().get_request(json_req::chain_get_finalized_head())?;
		match h {
			Some(hash) => Ok(Some(Runtime::Hash::from_hex(hash)?)),
			None => Ok(None),
		}
	}

	fn get_header(&self, hash: Option<Runtime::Hash>) -> ApiResult<Option<Runtime::Header>> {
		let h = self.client().get_request(json_req::chain_get_header(hash))?;
		match h {
			Some(hash) => Ok(Some(serde_json::from_str(&hash)?)),
			None => Ok(None),
		}
	}
}

impl<Signer, Client, Params, Runtime> GetBlock<Runtime::BlockNumber, Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Signer: Pair,
	MultiSignature: From<Signer::Signature>,
	Client: RpcClient,
	Runtime: FrameSystemConfig + GetRuntimeBlockType,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime::RuntimeBlock: DeserializeOwned,
	Runtime::Hash: FromHexString,
{
	type Block = Runtime::RuntimeBlock;

	fn get_block_hash(
		&self,
		number: Option<Runtime::BlockNumber>,
	) -> ApiResult<Option<Runtime::Hash>> {
		let h = self.client().get_request(json_req::chain_get_block_hash(number))?;
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
		let b = self.client().get_request(json_req::chain_get_block(hash))?;
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
