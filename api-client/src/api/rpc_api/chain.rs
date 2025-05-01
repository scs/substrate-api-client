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
use ac_primitives::config::Config;
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use alloc::vec::Vec;
use log::*;
use serde::de::DeserializeOwned;
use sp_runtime::generic::SignedBlock;

#[maybe_async::maybe_async(?Send)]
pub trait GetChainInfo {
	type BlockNumber;
	type Hash;
	type Header;
	type Block;

	async fn get_finalized_head(&self) -> Result<Option<Self::Hash>>;

	async fn get_header(&self, hash: Option<Self::Hash>) -> Result<Option<Self::Header>>;

	async fn get_block_hash(&self, number: Option<Self::BlockNumber>)
		-> Result<Option<Self::Hash>>;

	/// Returns the genesis block
	async fn get_genesis_block(&self) -> Result<Self::Block>;

	async fn get_block(&self, hash: Option<Self::Hash>) -> Result<Option<Self::Block>>;

	async fn get_block_by_num(
		&self,
		number: Option<Self::BlockNumber>,
	) -> Result<Option<Self::Block>>;

	/// A signed block is a block with Justification ,i.e., a Grandpa finality proof.
	/// The interval at which finality proofs are provided is set via the
	/// the `GrandpaConfig.justification_period` in a node's service.rs.
	/// The Justification may be None.
	async fn get_signed_block(
		&self,
		hash: Option<Self::Hash>,
	) -> Result<Option<SignedBlock<Self::Block>>>;

	async fn get_signed_block_by_num(
		&self,
		number: Option<Self::BlockNumber>,
	) -> Result<Option<SignedBlock<Self::Block>>>;

	/// Get the last finalized signed block.
	async fn get_finalized_block(&self) -> Result<Option<SignedBlock<Self::Block>>>;

	/// Returns a vector containing the blocks with the block numbers given in the input parameter.
	/// If fetching any of the block fails then a `Result::Err` will be returned.
	async fn get_signed_blocks(
		&self,
		block_numbers: &[Self::BlockNumber],
	) -> Result<Vec<SignedBlock<Self::Block>>>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> GetChainInfo for Api<T, Client>
where
	T: Config,
	Client: Request,
{
	type BlockNumber = T::BlockNumber;
	type Hash = T::Hash;
	type Header = T::Header;
	type Block = T::Block;

	async fn get_finalized_head(&self) -> Result<Option<Self::Hash>> {
		let finalized_block_hash =
			self.client().request("chain_getFinalizedHead", rpc_params![]).await?;
		Ok(finalized_block_hash)
	}

	async fn get_header(&self, hash: Option<Self::Hash>) -> Result<Option<Self::Header>> {
		let block_hash = self.client().request("chain_getHeader", rpc_params![hash]).await?;
		Ok(block_hash)
	}

	async fn get_block_hash(
		&self,
		number: Option<Self::BlockNumber>,
	) -> Result<Option<Self::Hash>> {
		let block_hash = self.client().request("chain_getBlockHash", rpc_params![number]).await?;
		Ok(block_hash)
	}

	async fn get_genesis_block(&self) -> Result<Self::Block> {
		self.get_block(Some(self.genesis_hash())).await?.ok_or(Error::BlockHashNotFound)
	}

	async fn get_block(&self, hash: Option<Self::Hash>) -> Result<Option<Self::Block>> {
		Self::get_signed_block(self, hash).await.map(|sb_opt| sb_opt.map(|sb| sb.block))
	}

	async fn get_block_by_num(
		&self,
		number: Option<Self::BlockNumber>,
	) -> Result<Option<Self::Block>> {
		Self::get_signed_block_by_num(self, number)
			.await
			.map(|sb_opt| sb_opt.map(|sb| sb.block))
	}

	async fn get_signed_block(
		&self,
		hash: Option<Self::Hash>,
	) -> Result<Option<SignedBlock<Self::Block>>> {
		let block = self.client().request("chain_getBlock", rpc_params![hash]).await?;
		Ok(block)
	}

	async fn get_signed_block_by_num(
		&self,
		number: Option<Self::BlockNumber>,
	) -> Result<Option<SignedBlock<Self::Block>>> {
		self.get_block_hash(number).await.map(|h| self.get_signed_block(h))?.await
	}

	async fn get_finalized_block(&self) -> Result<Option<SignedBlock<Self::Block>>> {
		let hash = self.get_finalized_head().await?;
		match hash {
			Some(hash) => self.get_signed_block(Some(hash)).await,
			None => Ok(None),
		}
	}

	async fn get_signed_blocks(
		&self,
		block_numbers: &[Self::BlockNumber],
	) -> Result<Vec<SignedBlock<Self::Block>>> {
		let mut blocks = Vec::<SignedBlock<Self::Block>>::new();

		for n in block_numbers {
			if let Some(block) = self.get_signed_block_by_num(Some(*n)).await? {
				blocks.push(block);
			}
		}
		Ok(blocks)
	}
}
#[maybe_async::maybe_async(?Send)]
pub trait SubscribeChain {
	type Client: Subscribe;
	type Header: DeserializeOwned;

	async fn subscribe_finalized_heads(
		&self,
	) -> Result<<Self::Client as Subscribe>::Subscription<Self::Header>>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> SubscribeChain for Api<T, Client>
where
	T: Config,
	Client: Subscribe,
{
	type Client = Client;
	type Header = T::Header;

	async fn subscribe_finalized_heads(
		&self,
	) -> Result<<Self::Client as Subscribe>::Subscription<Self::Header>> {
		debug!("subscribing to finalized heads");
		self.client()
			.subscribe(
				"chain_subscribeFinalizedHeads",
				rpc_params![],
				"chain_unsubscribeFinalizedHeads",
			)
			.await
			.map_err(|e| e.into())
	}
}
