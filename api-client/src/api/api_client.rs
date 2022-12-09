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
};
pub use frame_metadata::RuntimeMetadataPrefixed;
pub use serde_json::Value;
pub use sp_core::{crypto::Pair, storage::StorageKey};
pub use sp_runtime::{
	generic::{Block, SignedBlock},
	traits::{GetRuntimeBlockType, Header, IdentifyAccount},
	AccountId32, MultiSignature, MultiSigner,
};
pub use sp_std::prelude::*;

use crate::{rpc::Request, ReadProof};
use ac_compose_macros::rpc_params;
use ac_node_api::metadata::{Metadata, MetadataError};
use ac_primitives::{
	AccountInfo, BalancesConfig, ExtrinsicParams, FeeDetails, InclusionFee, RpcParams,
	RuntimeDispatchInfo,
};
use codec::{Decode, Encode};
use core::{
	convert::{TryFrom, TryInto},
	str::FromStr,
};
use log::{debug, info};
use serde::de::DeserializeOwned;
use sp_core::{storage::StorageData, Bytes};
use sp_rpc::number::NumberOrHex;
use sp_version::RuntimeVersion;
/// Api to talk with substrate-nodes
///
/// It is generic over the `RpcClient` trait, so you can use any rpc-backend you like.
///
/// # Custom Client Example
///
/// ```no_run
/// use substrate_api_client::{
///     Api, ApiClientError, ApiResult, FromHexString, Request, rpc::Error as RpcClientError, XtStatus, PlainTipExtrinsicParams, rpc::Result as RpcResult
/// };
/// use serde::de::DeserializeOwned;
/// use ac_primitives::RpcParams;
/// use kitchensink_runtime::Runtime;
/// use serde_json::{Value, json};
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
///         // Send json to node via web socket connection.
///         todo!()
///     }
/// }
///
/// impl Request for MyClient {
///     fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> Result<R, RpcClientError> {
///         let jsonreq = json!({
///         "method": method,
///         "params": params.to_json_value()?,
///         "jsonrpc": "2.0",
///         "id": "1",
///         });
///         let json_value = self.send_json::<Value>("".into(), jsonreq)?;
///         let value = serde_json::from_value(json_value)?;
///         Ok(value)
///     }
/// }
///
/// let client = MyClient::new();
/// let _api = Api::<(), _, PlainTipExtrinsicParams<Runtime>, Runtime>::new(client);
///
/// ```
#[derive(Clone)]
pub struct Api<Signer, Client, Params, Runtime>
where
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: BalancesConfig,
	Runtime::Hash: FromHexString,
	Runtime::Balance: TryFrom<NumberOrHex>,
{
	signer: Option<Signer>,
	genesis_hash: Runtime::Hash,
	metadata: Metadata,
	runtime_version: RuntimeVersion,
	client: Client,
	extrinsic_params_builder: Option<Params::OtherParams>,
}

impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Signer: Pair,
	MultiSigner: From<Signer::Public>,
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: BalancesConfig,
	Runtime::Hash: FromHexString,
	Runtime::Index: From<u32>,
	Runtime::Balance: TryFrom<NumberOrHex> + FromStr,
{
	/// Set the api signer account.
	pub fn set_signer(&mut self, signer: Signer) {
		self.signer = Some(signer);
	}

	/// Get the public part of the api signer account.
	pub fn signer_account(&self) -> Option<AccountId32> {
		let pair = self.signer.as_ref()?;
		let multi_signer = MultiSigner::from(pair.public());
		Some(multi_signer.into_account())
	}

	/// Get the private key pair of the api signer.
	pub fn signer(&self) -> Option<&Signer> {
		self.signer.as_ref()
	}

	/// Get the cached genesis hash of the substrate node.
	pub fn genesis_hash(&self) -> Runtime::Hash {
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
	pub fn extrinsic_params(&self, nonce: Runtime::Index) -> Params {
		let extrinsic_params_builder = self.extrinsic_params_builder.clone().unwrap_or_default();
		<Params as ExtrinsicParams<Runtime::Index, Runtime::Hash>>::new(
			self.runtime_version.spec_version,
			self.runtime_version.transaction_version,
			nonce,
			self.genesis_hash,
			extrinsic_params_builder,
		)
	}
}

impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Signer: Pair,
	MultiSigner: From<Signer::Public>,
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: GetRuntimeBlockType + BalancesConfig,
	Runtime::Hash: FromHexString,
	Runtime::Index: From<u32>,
	Runtime::Balance: TryFrom<NumberOrHex> + FromStr,
	Runtime::Header: DeserializeOwned,
	Runtime::RuntimeBlock: DeserializeOwned,
{
	/// Get nonce of signer account.
	pub fn get_nonce(&self) -> ApiResult<Runtime::Index> {
		if self.signer.is_none() {
			return Err(ApiClientError::NoSigner)
		}

		self.get_account_info(&self.signer_account().ok_or(ApiClientError::NoSigner)?)
			.map(|acc_opt| acc_opt.map_or_else(|| 0u32.into(), |acc| acc.nonce))
	}
}

/// Private node query methods. They should be used internally only, because the user should retrieve the data from the struct cache.
/// If an up-to-date query is necessary, cache should be updated beforehand.
impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: BalancesConfig,
	Runtime::Hash: FromHexString,
	Runtime::Balance: TryFrom<NumberOrHex> + FromStr,
	Runtime::Header: DeserializeOwned,
{
	fn get_genesis_hash(client: &Client) -> ApiResult<Runtime::Hash> {
		let genesis: Option<Runtime::Hash> =
			client.request("chain_getBlockHash", rpc_params![Some(0)])?;
		genesis.ok_or(ApiClientError::Genesis)
	}

	/// Get runtime version from node via websocket query.
	fn get_runtime_version(client: &Client) -> ApiResult<RuntimeVersion> {
		let version: RuntimeVersion = client.request("state_getRuntimeVersion", rpc_params![])?;
		Ok(version)
	}

	/// Get metadata from node via websocket query.
	fn get_metadata(client: &Client) -> ApiResult<Metadata> {
		let metadata_bytes: Bytes = client.request("state_getMetadata", rpc_params![])?;

		let metadata = RuntimeMetadataPrefixed::decode(&mut metadata_bytes.0.as_slice())?;
		Metadata::try_from(metadata).map_err(|e| e.into())
	}
}

/// Substrate node calls via websocket.
impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: GetRuntimeBlockType + BalancesConfig,
	Runtime::Hash: FromHexString,
	Runtime::Index: From<u32>,
	Runtime::Balance: TryFrom<NumberOrHex> + FromStr,
	Runtime::Header: DeserializeOwned,
	Runtime::RuntimeBlock: DeserializeOwned,
{
	pub fn new(client: Client) -> ApiResult<Self> {
		let genesis_hash = Self::get_genesis_hash(&client)?;
		info!("Got genesis hash: {:?}", genesis_hash);

		let metadata = Self::get_metadata(&client)?;
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
		let metadata = Self::get_metadata(&self.client)?;
		debug!("Metadata: {:?}", metadata);

		let runtime_version = Self::get_runtime_version(&self.client)?;
		info!("Runtime Version: {:?}", runtime_version);

		self.metadata = metadata;
		self.runtime_version = runtime_version;
		Ok(())
	}

	pub fn get_account_info<AccountId: Clone + Encode>(
		&self,
		address: &AccountId,
	) -> ApiResult<Option<AccountInfo<Runtime::Index, Runtime::AccountData>>> {
		let storagekey: sp_core::storage::StorageKey =
			self.metadata
				.storage_map_key::<AccountId>("System", "Account", address.clone())?;

		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_by_key_hash(storagekey, None)
	}

	pub fn get_account_data(
		&self,
		address: &Runtime::AccountId,
	) -> ApiResult<Option<Runtime::AccountData>> {
		self.get_account_info(address).map(|info| info.map(|i| i.data))
	}

	pub fn get_finalized_head(&self) -> ApiResult<Option<Runtime::Hash>> {
		let finalized_block_hash = self.request("chain_getFinalizedHead", rpc_params![])?;
		Ok(finalized_block_hash)
	}

	pub fn get_header(&self, hash: Option<Runtime::Hash>) -> ApiResult<Option<Runtime::Header>> {
		let block_hash = self.request("chain_getHeader", rpc_params![hash])?;
		Ok(block_hash)
	}

	pub fn get_block_hash(
		&self,
		number: Option<Runtime::BlockNumber>,
	) -> ApiResult<Option<Runtime::Hash>> {
		let block_hash = self.request("chain_getBlockHash", rpc_params![number])?;
		Ok(block_hash)
	}

	pub fn get_block(
		&self,
		hash: Option<Runtime::Hash>,
	) -> ApiResult<Option<Runtime::RuntimeBlock>> {
		Self::get_signed_block(self, hash).map(|sb_opt| sb_opt.map(|sb| sb.block))
	}

	pub fn get_block_by_num(
		&self,
		number: Option<Runtime::BlockNumber>,
	) -> ApiResult<Option<Runtime::RuntimeBlock>> {
		Self::get_signed_block_by_num(self, number).map(|sb_opt| sb_opt.map(|sb| sb.block))
	}

	/// A signed block is a block with Justification ,i.e., a Grandpa finality proof.
	/// The interval at which finality proofs are provided is set via the
	/// the `GrandpaConfig.justification_period` in a node's service.rs.
	/// The Justification may be none.
	pub fn get_signed_block(
		&self,
		hash: Option<Runtime::Hash>,
	) -> ApiResult<Option<SignedBlock<Runtime::RuntimeBlock>>> {
		let block = self.request("chain_getBlock", rpc_params![hash])?;
		Ok(block)
	}

	pub fn get_signed_block_by_num(
		&self,
		number: Option<Runtime::BlockNumber>,
	) -> ApiResult<Option<SignedBlock<Runtime::RuntimeBlock>>> {
		self.get_block_hash(number).map(|h| self.get_signed_block(h))?
	}

	pub fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> ApiResult<R> {
		self.client.request(method, params).map_err(ApiClientError::RpcClient)
	}

	pub fn get_storage_value<V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Runtime::Hash>,
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
		at_block: Option<Runtime::Hash>,
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
		at_block: Option<Runtime::Hash>,
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
		at_block: Option<Runtime::Hash>,
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
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<Vec<u8>>> {
		let storage: Option<StorageData> =
			self.request("state_getStorage", rpc_params![key, at_block])?;
		Ok(storage.map(|storage_data| storage_data.0))
	}

	pub fn get_storage_value_proof(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<ReadProof<Runtime::Hash>>> {
		let storagekey = self.metadata.storage_value_key(storage_prefix, storage_key_name)?;
		info!("storage key is: 0x{}", hex::encode(&storagekey));
		self.get_storage_proof_by_keys(vec![storagekey], at_block)
	}

	pub fn get_storage_map_proof<K: Encode, V: Decode + Clone>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		map_key: K,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<ReadProof<Runtime::Hash>>> {
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
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<ReadProof<Runtime::Hash>>> {
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
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<ReadProof<Runtime::Hash>>> {
		let proof = self.request("state_getReadProof", rpc_params![keys, at_block])?;
		Ok(proof)
	}

	pub fn get_keys(
		&self,
		key: StorageKey,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<Vec<String>>> {
		let keys = self.request("state_getKeys", rpc_params![key, at_block])?;
		Ok(keys)
	}

	pub fn get_fee_details(
		&self,
		xthex_prefixed: &str,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<FeeDetails<Runtime::Balance>>> {
		let details: Option<FeeDetails<NumberOrHex>> =
			self.request("payment_queryFeeDetails", rpc_params![xthex_prefixed, at_block])?;

		let details = match details {
			Some(details) => Some(convert_fee_details(details)?),
			None => None,
		};
		Ok(details)
	}

	pub fn get_payment_info(
		&self,
		xthex_prefixed: &str,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<RuntimeDispatchInfo<Runtime::Balance>>> {
		let res = self.request("payment_queryInfo", rpc_params![xthex_prefixed, at_block])?;
		Ok(res)
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

	pub fn get_existential_deposit(&self) -> ApiResult<Runtime::Balance> {
		self.get_constant("Balances", "ExistentialDeposit")
	}

	/// Submit an extrsinic to the substrate node, without watching.
	pub fn submit_extrinsic(&self, xthex_prefixed: String) -> ApiResult<Runtime::Hash> {
		debug!("sending extrinsic: {:?}", xthex_prefixed);
		let xt_hash = self.client.request("author_submitExtrinsic", rpc_params![xthex_prefixed])?;
		Ok(xt_hash)
	}
}

fn convert_fee_details<Balance: TryFrom<NumberOrHex>>(
	details: FeeDetails<NumberOrHex>,
) -> ApiResult<FeeDetails<Balance>> {
	let inclusion_fee = if let Some(inclusion_fee) = details.inclusion_fee {
		Some(inclusion_fee_with_balance(inclusion_fee)?)
	} else {
		None
	};
	let tip = details.tip.try_into().map_err(|_| ApiClientError::TryFromIntError)?;
	Ok(FeeDetails { inclusion_fee, tip })
}

fn inclusion_fee_with_balance<Balance: TryFrom<NumberOrHex>>(
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
