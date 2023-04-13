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
};
use ac_compose_macros::rpc_params;
use ac_primitives::{config::Config, serde_impls::SignedBlock};
use log::*;
use serde::{de::DeserializeOwned, Serialize};

pub trait GetHeader<Hash> {
	type Header;

	fn get_finalized_head(&self) -> Result<Option<Hash>>;

	fn get_header(&self, hash: Option<Hash>) -> Result<Option<Self::Header>>;
}

impl<T: Config, Client> GetHeader<T::Hash> for Api<T, Client>
where
	Client: Request,
{
	type Header = T::Header;

	fn get_finalized_head(&self) -> Result<Option<T::Hash>> {
		let finalized_block_hash =
			self.client().request("chain_getFinalizedHead", rpc_params![])?;
		Ok(finalized_block_hash)
	}

	fn get_header(&self, hash: Option<T::Hash>) -> Result<Option<T::Header>> {
		let block_hash = self.client().request("chain_getHeader", rpc_params![hash])?;
		Ok(block_hash)
	}
}

pub trait GetBlock<Number, Hash> {
	type Block;

	fn get_block_hash(&self, number: Option<Number>) -> Result<Option<Hash>>;

	fn get_block(&self, hash: Option<Hash>) -> Result<Option<Self::Block>>;

	fn get_block_by_num(&self, number: Option<Number>) -> Result<Option<Self::Block>>;

	/// A signed block is a block with Justification ,i.e., a Grandpa finality proof.
	/// The interval at which finality proofs are provided is set via the
	/// the `GrandpaConfig.justification_period` in a node's service.rs.
	/// The Justification may be None.
	fn get_signed_block(&self, hash: Option<Hash>) -> Result<Option<SignedBlock<Self::Block>>>;

	fn get_signed_block_by_num(
		&self,
		number: Option<Number>,
	) -> Result<Option<SignedBlock<Self::Block>>>;
}
impl<T: Config, Client> GetBlock<<T::Header as ac_primitives::config::Header>::Number, T::Hash>
	for Api<T, Client>
where
	Client: Request,
	<T::Header as ac_primitives::config::Header>::Number: Serialize,
{
	type Block = T::Block;

	fn get_block_hash(
		&self,
		number: Option<<T::Header as ac_primitives::config::Header>::Number>,
	) -> Result<Option<T::Hash>> {
		let block_hash = self.client().request("chain_getBlockHash", rpc_params![number])?;
		Ok(block_hash)
	}

	fn get_block(&self, hash: Option<T::Hash>) -> Result<Option<Self::Block>> {
		Self::get_signed_block(self, hash).map(|sb_opt| sb_opt.map(|sb| sb.block))
	}

	fn get_block_by_num(
		&self,
		number: Option<<T::Header as ac_primitives::config::Header>::Number>,
	) -> Result<Option<Self::Block>> {
		Self::get_signed_block_by_num(self, number).map(|sb_opt| sb_opt.map(|sb| sb.block))
	}

	fn get_signed_block(&self, hash: Option<T::Hash>) -> Result<Option<SignedBlock<Self::Block>>> {
		let block = self.client().request("chain_getBlock", rpc_params![hash])?;
		Ok(block)
	}

	fn get_signed_block_by_num(
		&self,
		number: Option<<T::Header as ac_primitives::config::Header>::Number>,
	) -> Result<Option<SignedBlock<Self::Block>>> {
		self.get_block_hash(number).map(|h| self.get_signed_block(h))?
	}
}
pub trait SubscribeChain<Client, Hash>
where
	Client: Subscribe,
	Hash: DeserializeOwned,
{
	type Header: DeserializeOwned;

	fn subscribe_finalized_heads(&self) -> Result<Client::Subscription<Self::Header>>;
}

impl<T: Config, Client> SubscribeChain<Client, T::Hash> for Api<T, Client>
where
	Client: Subscribe,
	<T::Header as ac_primitives::config::Header>::Number: Serialize,
{
	type Header = T::Header;

	fn subscribe_finalized_heads(&self) -> Result<Client::Subscription<Self::Header>> {
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
