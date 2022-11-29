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

pub use crate::{
	api::{
		error::{ApiResult, Error as ApiClientError},
		XtStatus,
	},
	utils::FromHexString,
	Hash, Index,
};
pub use frame_metadata::RuntimeMetadataPrefixed;
pub use pallet_transaction_payment::FeeDetails;
pub use serde_json::Value;
pub use sp_core::{crypto::Pair, storage::StorageKey};
pub use sp_runtime::{
	generic::SignedBlock,
	traits::{Block, Header, IdentifyAccount},
	AccountId32 as AccountId, MultiSignature, MultiSigner,
};
pub use sp_std::prelude::*;
pub use sp_version::RuntimeVersion;

use crate::{
	rpc::{json_req, RpcClient},
	ReadProof,
};
use ac_node_api::metadata::{Metadata, MetadataError};
use ac_primitives::{AccountData, AccountInfo, Balance, ExtrinsicParams};
use codec::{Decode, Encode};
use log::{debug, info};
use pallet_transaction_payment::{InclusionFee, RuntimeDispatchInfo};
use serde::de::DeserializeOwned;
use sp_rpc::number::NumberOrHex;
use std::convert::{TryFrom, TryInto};

/// Api to talk with substrate-nodes
///
/// It is generic over the `RpcClient` trait, so you can use any rpc-backend you like.
///
/// # Custom Client Example
///
/// ```no_run
/// use substrate_api_client::rpc::json_req::author_submit_extrinsic;
/// use substrate_api_client::{
///     Api, ApiClientError, ApiResult, FromHexString, Hash, RpcClient, rpc::Error as RpcClientError,  XtStatus, PlainTipExtrinsicParams, rpc::Result as RpcResult
/// };
/// use serde_json::Value;
/// struct MyClient {
///     // pick any request crate, such as ureq::Agent
///     _inner: (),
/// }
///
/// impl MyClient {
///     pub fn new() -> Self {
///         Self {
///             // ureq::agent()
///             _inner: (),
///         }
///     }
///
///     pub fn send_json<R>(
///         &self,
///         _path: String,
///         _json: Value,
///     ) -> Result<R, RpcClientError> {
///         // you can figure this out...self.inner...send_json...
///         todo!()
///     }
/// }
///
/// impl RpcClient for MyClient {
///     fn get_request(&self, jsonreq: Value) -> RpcResult<Option<String>> {
///         self.send_json::<Value>("".into(), jsonreq)
///             .map(|v| Some(v.to_string()))
///     }
///
///     fn send_extrinsic(
///         &self,
///         xthex_prefixed: String,
///         _exit_on: XtStatus,
///     ) -> RpcResult<Option<Hash>> {
///         let jsonreq = author_submit_extrinsic(&xthex_prefixed);
///         let res: String = self
///             .send_json("".into(), jsonreq)?;
///         Ok(Some(Hash::from_hex(res)?))
///     }
/// }
///
/// let client = MyClient::new();
/// let _api = Api::<(), _, PlainTipExtrinsicParams>::new(client);
///
/// ```
#[derive(Clone)]
pub struct Api<P, Client, Params>
where
	Client: RpcClient,
	Params: ExtrinsicParams<Index, Hash>,
{
	signer: Option<P>,
	genesis_hash: Hash,
	metadata: Metadata,
	runtime_version: RuntimeVersion,
	client: Client,
	extrinsic_params_builder: Option<Params::OtherParams>,
}

/// Setter and getter calls to the local cache, no substrate node calls involved.
impl<P, Client, Params> Api<P, Client, Params>
where
	P: Pair,
	MultiSigner: From<P::Public>,
	Client: RpcClient,
	Params: ExtrinsicParams<Index, Hash>,
{
	/// Set the api signer account.
	pub fn set_signer(&mut self, signer: P) {
		self.signer = Some(signer);
	}

	/// Get the public part of the api signer account.
	pub fn signer_account(&self) -> Option<AccountId> {
		let pair = self.signer.as_ref()?;
		let multi_signer = MultiSigner::from(pair.public());
		Some(multi_signer.into_account())
	}

	/// Get the private key pair of the api signer.
	pub fn signer(&self) -> Option<&P> {
		self.signer.as_ref()
	}

	/// Get the cached genesis hash of the substrate node.
	pub fn genesis_hash(&self) -> Hash {
		self.genesis_hash
	}

	/// Get the cached metadata of the substrate node.
	pub fn metadata(&self) -> &Metadata {
		&self.metadata
	}

	/// Get the cached runtime version of the substrate node.
	pub fn runtime_version(&self) -> &RuntimeVersion {
		&self.runtime_version
	}

	/// Get the cached spec version of the substrate node.
	pub fn spec_version(&self) -> u32 {
		self.runtime_version.spec_version
	}

	/// Get the rpc client.
	pub fn client(&self) -> &Client {
		&self.client
	}

	/// Set the extrinscs param builder.
	pub fn set_extrinsic_params_builder(&mut self, extrinsic_params: Params::OtherParams) {
		self.extrinsic_params_builder = Some(extrinsic_params);
	}

	/// Get the extrinsic params, built with the set or if none, the default Params Builder.
	pub fn extrinsic_params(&self, nonce: Index) -> Params {
		let extrinsic_params_builder = self.extrinsic_params_builder.clone().unwrap_or_default();
		<Params as ExtrinsicParams<Index, Hash>>::new(
			self.runtime_version.spec_version,
			self.runtime_version.transaction_version,
			nonce,
			self.genesis_hash,
			extrinsic_params_builder,
		)
	}
}

impl<P, Client, Params> Api<P, Client, Params>
where
	P: Pair,
	MultiSigner: From<P::Public>,
	Client: RpcClient,
	Params: ExtrinsicParams<Index, Hash>,
{
	/// Get nonce of signer account.
	pub fn get_nonce(&self) -> ApiResult<u32> {
		if self.signer.is_none() {
			return Err(ApiClientError::NoSigner)
		}

		self.get_account_info(&self.signer_account().unwrap())
			.map(|acc_opt| acc_opt.map_or_else(|| 0, |acc| acc.nonce))
	}
}

/// Private node query methods. They should be used internally only, because the user should retrieve the data from the struct cache.
/// If an up-to-date query is necessary, cache should be updated beforehand.
impl<P, Client, Params> Api<P, Client, Params>
where
	Client: RpcClient,
	Params: ExtrinsicParams<Index, Hash>,
{
	/// Get genesis hash from node via websocket query.
	fn get_genesis_hash(client: &Client) -> ApiResult<Hash> {
		let jsonreq = json_req::chain_get_genesis_hash();
		let genesis = client.get_request(jsonreq)?;

		match genesis {
			Some(g) => Hash::from_hex(g).map_err(|e| e.into()),
			None => Err(ApiClientError::Genesis),
		}
	}

	/// Get runtime version from node via websocket query.
	fn get_runtime_version(client: &Client) -> ApiResult<RuntimeVersion> {
		let jsonreq = json_req::state_get_runtime_version();
		let version = client.get_request(jsonreq)?;

		match version {
			Some(v) => serde_json::from_str(&v).map_err(|e| e.into()),
			None => Err(ApiClientError::RuntimeVersion),
		}
	}

	/// Get metadata from node via websocket query.
	fn get_metadata(client: &Client) -> ApiResult<RuntimeMetadataPrefixed> {
		let jsonreq = json_req::state_get_metadata();
		let meta = client.get_request(jsonreq)?;

		if meta.is_none() {
			return Err(ApiClientError::MetadataFetch)
		}
		let metadata = Vec::from_hex(meta.unwrap())?;
		RuntimeMetadataPrefixed::decode(&mut metadata.as_slice()).map_err(|e| e.into())
	}
}

/// Substrate node calls via websocket.
impl<P, Client, Params> Api<P, Client, Params>
where
	Client: RpcClient,
	Params: ExtrinsicParams<Index, Hash>,
{
	pub fn new(client: Client) -> ApiResult<Self> {
		let genesis_hash = Self::get_genesis_hash(&client)?;
		info!("Got genesis hash: {:?}", genesis_hash);

		let metadata = Self::get_metadata(&client).map(Metadata::try_from)??;
		debug!("Metadata: {:?}", metadata);

		let runtime_version = Self::get_runtime_version(&client)?;
		info!("Runtime Version: {:?}", runtime_version);

		Ok(Self {
			signer: None,
			genesis_hash,
			metadata,
			runtime_version,
			client,
			extrinsic_params_builder: None,
		})
	}

	/// Updates the runtime and metadata of the api via node query.
	// Ideally, this function is called if a substrate update runtime event is encountered.
	pub fn update_runtime(&mut self) -> ApiResult<()> {
		let metadata = Self::get_metadata(&self.client).map(Metadata::try_from)??;
		debug!("Metadata: {:?}", metadata);

		let runtime_version = Self::get_runtime_version(&self.client)?;
		info!("Runtime Version: {:?}", runtime_version);

		self.metadata = metadata;
		self.runtime_version = runtime_version;
		Ok(())
	}

	pub fn get_account_info(&self, address: &AccountId) -> ApiResult<Option<AccountInfo>> {
		let storagekey: sp_core::storage::StorageKey =
			self.metadata
				.storage_map_key::<AccountId>("System", "Account", address.clone())?;

		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key_hash(storagekey, None)
	}

	pub fn get_account_data(&self, address: &AccountId) -> ApiResult<Option<AccountData>> {
		self.get_account_info(address).map(|info| info.map(|i| i.data))
	}

	pub fn get_finalized_head(&self) -> ApiResult<Option<Hash>> {
		let h = self.get_request(json_req::chain_get_finalized_head())?;
		match h {
			Some(hash) => Ok(Some(Hash::from_hex(hash)?)),
			None => Ok(None),
		}
	}

	pub fn get_header<H>(&self, hash: Option<Hash>) -> ApiResult<Option<H>>
	where
		H: Header + DeserializeOwned,
	{
		let h = self.get_request(json_req::chain_get_header(hash))?;
		match h {
			Some(hash) => Ok(Some(serde_json::from_str(&hash)?)),
			None => Ok(None),
		}
	}

	pub fn get_block_hash(&self, number: Option<u32>) -> ApiResult<Option<Hash>> {
		let h = self.get_request(json_req::chain_get_block_hash(number))?;
		match h {
			Some(hash) => Ok(Some(Hash::from_hex(hash)?)),
			None => Ok(None),
		}
	}

	pub fn get_block<B>(&self, hash: Option<Hash>) -> ApiResult<Option<B>>
	where
		B: Block + DeserializeOwned,
	{
		Self::get_signed_block(self, hash).map(|sb_opt| sb_opt.map(|sb| sb.block))
	}

	pub fn get_block_by_num<B>(&self, number: Option<u32>) -> ApiResult<Option<B>>
	where
		B: Block + DeserializeOwned,
	{
		Self::get_signed_block_by_num(self, number).map(|sb_opt| sb_opt.map(|sb| sb.block))
	}

	/// A signed block is a block with Justification ,i.e., a Grandpa finality proof.
	/// The interval at which finality proofs are provided is set via the
	/// the `GrandpaConfig.justification_period` in a node's service.rs.
	/// The Justification may be none.
	pub fn get_signed_block<B>(&self, hash: Option<Hash>) -> ApiResult<Option<SignedBlock<B>>>
	where
		B: Block + DeserializeOwned,
	{
		let b = self.get_request(json_req::chain_get_block(hash))?;
		match b {
			Some(block) => Ok(Some(serde_json::from_str(&block)?)),
			None => Ok(None),
		}
	}

	pub fn get_signed_block_by_num<B>(
		&self,
		number: Option<u32>,
	) -> ApiResult<Option<SignedBlock<B>>>
	where
		B: Block + DeserializeOwned,
	{
		self.get_block_hash(number).map(|h| self.get_signed_block(h))?
	}

	pub fn get_request(&self, jsonreq: Value) -> ApiResult<Option<String>> {
		self.client.get_request(jsonreq).map_err(ApiClientError::RpcClient)
	}

	pub fn get_storage_value<V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Hash>,
	) -> ApiResult<Option<V>> {
		let storagekey = self.metadata.storage_value_key(storage_prefix, storage_key_name)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key_hash(storagekey, at_block)
	}

	pub fn get_storage_map<K: Encode, V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		map_key: K,
		at_block: Option<Hash>,
	) -> ApiResult<Option<V>> {
		let storagekey =
			self.metadata.storage_map_key::<K>(storage_prefix, storage_key_name, map_key)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key_hash(storagekey, at_block)
	}

	pub fn get_storage_map_key_prefix(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
	) -> ApiResult<StorageKey> {
		self.metadata
			.storage_map_key_prefix(storage_prefix, storage_key_name)
			.map_err(|e| e.into())
	}

	pub fn get_storage_double_map<K: Encode, Q: Encode, V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
		second: Q,
		at_block: Option<Hash>,
	) -> ApiResult<Option<V>> {
		let storagekey = self.metadata.storage_double_map_key::<K, Q>(
			storage_prefix,
			storage_key_name,
			first,
			second,
		)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key_hash(storagekey, at_block)
	}

	pub fn get_storage_by_key_hash<V: Decode>(
		&self,
		key: StorageKey,
		at_block: Option<Hash>,
	) -> ApiResult<Option<V>> {
		let s = self.get_opaque_storage_by_key_hash(key, at_block)?;
		match s {
			Some(storage) => Ok(Some(Decode::decode(&mut storage.as_slice())?)),
			None => Ok(None),
		}
	}

	pub fn get_opaque_storage_by_key_hash(
		&self,
		key: StorageKey,
		at_block: Option<Hash>,
	) -> ApiResult<Option<Vec<u8>>> {
		let jsonreq = json_req::state_get_storage(key, at_block);
		let s = self.get_request(jsonreq)?;

		match s {
			Some(storage) => Ok(Some(Vec::from_hex(storage)?)),
			None => Ok(None),
		}
	}

	pub fn get_storage_value_proof(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Hash>,
	) -> ApiResult<Option<ReadProof<Hash>>> {
		let storagekey = self.metadata.storage_value_key(storage_prefix, storage_key_name)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_proof_by_keys(vec![storagekey], at_block)
	}

	pub fn get_storage_map_proof<K: Encode, V: Decode + Clone>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		map_key: K,
		at_block: Option<Hash>,
	) -> ApiResult<Option<ReadProof<Hash>>> {
		let storagekey =
			self.metadata.storage_map_key::<K>(storage_prefix, storage_key_name, map_key)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_proof_by_keys(vec![storagekey], at_block)
	}

	pub fn get_storage_double_map_proof<K: Encode, Q: Encode, V: Decode + Clone>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
		second: Q,
		at_block: Option<Hash>,
	) -> ApiResult<Option<ReadProof<Hash>>> {
		let storagekey = self.metadata.storage_double_map_key::<K, Q>(
			storage_prefix,
			storage_key_name,
			first,
			second,
		)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_proof_by_keys(vec![storagekey], at_block)
	}

	pub fn get_storage_proof_by_keys(
		&self,
		keys: Vec<StorageKey>,
		at_block: Option<Hash>,
	) -> ApiResult<Option<ReadProof<Hash>>> {
		let jsonreq = json_req::state_get_read_proof(keys, at_block);
		let p = self.get_request(jsonreq)?;
		match p {
			Some(proof) => Ok(Some(serde_json::from_str(&proof)?)),
			None => Ok(None),
		}
	}

	pub fn get_keys(
		&self,
		key: StorageKey,
		at_block: Option<Hash>,
	) -> ApiResult<Option<Vec<String>>> {
		let jsonreq = json_req::state_get_keys(key, at_block);
		let k = self.get_request(jsonreq)?;
		match k {
			Some(keys) => Ok(Some(serde_json::from_str(&keys)?)),
			None => Ok(None),
		}
	}

	pub fn get_fee_details(
		&self,
		xthex_prefixed: &str,
		at_block: Option<Hash>,
	) -> ApiResult<Option<FeeDetails<Balance>>> {
		let jsonreq = json_req::payment_query_fee_details(xthex_prefixed, at_block);
		let res = self.get_request(jsonreq)?;
		match res {
			Some(details) => {
				let details: FeeDetails<NumberOrHex> = serde_json::from_str(&details)?;
				let details = convert_fee_details(details)?;
				Ok(Some(details))
			},
			None => Ok(None),
		}
	}

	pub fn get_payment_info(
		&self,
		xthex_prefixed: &str,
		at_block: Option<Hash>,
	) -> ApiResult<Option<RuntimeDispatchInfo<Balance>>> {
		let jsonreq = json_req::payment_query_info(xthex_prefixed, at_block);
		let res = self.get_request(jsonreq)?;
		match res {
			Some(info) => {
				let info: RuntimeDispatchInfo<Balance> = serde_json::from_str(&info)?;
				Ok(Some(info))
			},
			None => Ok(None),
		}
	}

	pub fn get_constant<C: Decode>(
		&self,
		pallet: &'static str,
		constant: &'static str,
	) -> ApiResult<C> {
		let c = self
			.metadata
			.pallet(pallet)?
			.constants
			.get(constant)
			.ok_or(MetadataError::ConstantNotFound(constant))?;

		Ok(Decode::decode(&mut c.value.as_slice())?)
	}

	pub fn get_existential_deposit(&self) -> ApiResult<Balance> {
		self.get_constant("Balances", "ExistentialDeposit")
	}

	#[cfg(feature = "ws-client")]
	pub fn send_extrinsic(
		&self,
		xthex_prefixed: String,
		exit_on: XtStatus,
	) -> ApiResult<Option<Hash>> {
		debug!("sending extrinsic: {:?}", xthex_prefixed);
		self.client
			.send_extrinsic(xthex_prefixed, exit_on)
			.map_err(ApiClientError::RpcClient)
	}

	#[cfg(not(feature = "ws-client"))]
	pub fn send_extrinsic(&self, xthex_prefixed: String) -> ApiResult<Option<Hash>> {
		debug!("sending extrinsic: {:?}", xthex_prefixed);
		// XtStatus should never be used used but we need to put something
		self.client.send_extrinsic(xthex_prefixed, XtStatus::Broadcast)
	}
}

fn convert_fee_details(details: FeeDetails<NumberOrHex>) -> ApiResult<FeeDetails<u128>> {
	let inclusion_fee = if let Some(inclusion_fee) = details.inclusion_fee {
		Some(inclusion_fee_with_balance(inclusion_fee)?)
	} else {
		None
	};
	let tip = details.tip.try_into().map_err(|_| ApiClientError::TryFromIntError)?;
	Ok(FeeDetails { inclusion_fee, tip })
}

fn inclusion_fee_with_balance(
	inclusion_fee: InclusionFee<NumberOrHex>,
) -> ApiResult<InclusionFee<Balance>> {
	Ok(InclusionFee {
		base_fee: inclusion_fee.base_fee.try_into().map_err(|_| ApiClientError::TryFromIntError)?,
		len_fee: inclusion_fee.len_fee.try_into().map_err(|_| ApiClientError::TryFromIntError)?,
		adjusted_weight_fee: inclusion_fee
			.adjusted_weight_fee
			.try_into()
			.map_err(|_| ApiClientError::TryFromIntError)?,
	})
}
