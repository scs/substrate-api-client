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
	api::{Api, Result},
	rpc::{Request, Subscribe},
	Error,
};
use ac_compose_macros::rpc_params;
use ac_primitives::{ExtrinsicParams, FrameSystemConfig, SignedBlock};
use alloc::vec::Vec;
use log::*;
use serde::de::DeserializeOwned;
use sp_runtime::traits::GetRuntimeBlockType;

pub trait GetChainInfo {
	type BlockNumber;
	type Hash;
	type Header;
	type Block;

	fn get_finalized_head(&self) -> Result<Option<Self::Hash>>;

	fn get_header(&self, hash: Option<Self::Hash>) -> Result<Option<Self::Header>>;

	fn get_block_hash(&self, number: Option<Self::BlockNumber>) -> Result<Option<Self::Hash>>;

	/// Returns the genesis block
	fn get_genesis_block(&self) -> Result<Self::Block>;

	fn get_block(&self, hash: Option<Self::Hash>) -> Result<Option<Self::Block>>;

	fn get_block_by_num(&self, number: Option<Self::BlockNumber>) -> Result<Option<Self::Block>>;

	/// A signed block is a block with Justification ,i.e., a Grandpa finality proof.
	/// The interval at which finality proofs are provided is set via the
	/// the `GrandpaConfig.justification_period` in a node's service.rs.
	/// The Justification may be None.
	fn get_signed_block(
		&self,
		hash: Option<Self::Hash>,
	) -> Result<Option<SignedBlock<Self::Block>>>;

	fn get_signed_block_by_num(
		&self,
		number: Option<Self::BlockNumber>,
	) -> Result<Option<SignedBlock<Self::Block>>>;

	/// Get the last finalized signed block.
	fn get_finalized_block(&self) -> Result<Option<SignedBlock<Self::Block>>>;

	/// Returns a vector containing the blocks with the block numbers given in the input parameter.
	/// If fetching any of the block fails then a `Result::Err` will be returned.
	fn get_signed_blocks(
		&self,
		block_numbers: &[Self::BlockNumber],
	) -> Result<Vec<SignedBlock<Self::Block>>>;
}

impl<Signer, Client, Params, Runtime> GetChainInfo for Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Runtime: FrameSystemConfig + GetRuntimeBlockType,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime::RuntimeBlock: DeserializeOwned,
	Runtime::Header: DeserializeOwned,
{
	type BlockNumber = Runtime::BlockNumber;
	type Hash = Runtime::Hash;
	type Header = Runtime::Header;
	type Block = Runtime::RuntimeBlock;

	fn get_finalized_head(&self) -> Result<Option<Self::Hash>> {
		let finalized_block_hash =
			self.client().request("chain_getFinalizedHead", rpc_params![])?;
		Ok(finalized_block_hash)
	}

	fn get_header(&self, hash: Option<Self::Hash>) -> Result<Option<Self::Header>> {
		let block_hash = self.client().request("chain_getHeader", rpc_params![hash])?;
		Ok(block_hash)
	}

	fn get_block_hash(&self, number: Option<Self::BlockNumber>) -> Result<Option<Self::Hash>> {
		let block_hash = self.client().request("chain_getBlockHash", rpc_params![number])?;
		Ok(block_hash)
	}

	fn get_genesis_block(&self) -> Result<Self::Block> {
		self.get_block(Some(self.genesis_hash()))?.ok_or(Error::BlockHashNotFound)
	}

	fn get_block(&self, hash: Option<Self::Hash>) -> Result<Option<Self::Block>> {
		Self::get_signed_block(self, hash).map(|sb_opt| sb_opt.map(|sb| sb.block))
	}

	fn get_block_by_num(&self, number: Option<Self::BlockNumber>) -> Result<Option<Self::Block>> {
		Self::get_signed_block_by_num(self, number).map(|sb_opt| sb_opt.map(|sb| sb.block))
	}

	fn get_signed_block(
		&self,
		hash: Option<Self::Hash>,
	) -> Result<Option<SignedBlock<Self::Block>>> {
		let block = self.client().request("chain_getBlock", rpc_params![hash])?;
		Ok(block)
	}

	fn get_signed_block_by_num(
		&self,
		number: Option<Self::BlockNumber>,
	) -> Result<Option<SignedBlock<Self::Block>>> {
		self.get_block_hash(number).map(|h| self.get_signed_block(h))?
	}

	fn get_finalized_block(&self) -> Result<Option<SignedBlock<Self::Block>>> {
		self.get_finalized_head()?
			.map_or_else(|| Ok(None), |hash| self.get_signed_block(Some(hash)))
	}

	fn get_signed_blocks(
		&self,
		block_numbers: &[Self::BlockNumber],
	) -> Result<Vec<SignedBlock<Self::Block>>> {
		let mut blocks = Vec::<SignedBlock<Self::Block>>::new();

		for n in block_numbers {
			if let Some(block) = self.get_signed_block_by_num(Some(*n))? {
				blocks.push(block);
			}
		}
		Ok(blocks)
	}
}
pub trait SubscribeChain {
	type Client: Subscribe;
	type Header: DeserializeOwned;

	fn subscribe_finalized_heads(
		&self,
	) -> Result<<Self::Client as Subscribe>::Subscription<Self::Header>>;
}

impl<Signer, Client, Params, Runtime> SubscribeChain for Api<Signer, Client, Params, Runtime>
where
	Client: Subscribe,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
	Runtime::Header: DeserializeOwned,
{
	type Client = Client;
	type Header = Runtime::Header;

	fn subscribe_finalized_heads(
		&self,
	) -> Result<<Self::Client as Subscribe>::Subscription<Self::Header>> {
		debug!("subscribing to finalized heads");
		self.client()
			.subscribe(
				"chain_subscribeFinalizedHeads",
				rpc_params![],
				"chain_unsubscribeFinalizedHeads",
			)
			.map_err(|e| e.into())
	}
}
