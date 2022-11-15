use crate::rpc::json_req;
pub use crate::{
	api_client::{
		error::{ApiResult, Error as ApiClientError},
		rpc::XtStatus,
	},
	utils::FromHexString,
};
use ac_node_api::metadata::{Metadata, MetadataError};
use ac_primitives::{AccountData, AccountInfo, Balance, ExtrinsicParams};
use codec::{Decode, Encode};
use log::{debug, info};
pub use metadata::RuntimeMetadataPrefixed;
use serde::de::DeserializeOwned;
pub use serde_json::Value;
use sp_core::H256 as Hash;
pub use sp_core::{crypto::Pair, storage::StorageKey};
use sp_rpc::number::NumberOrHex;
pub use sp_runtime::{
	generic::SignedBlock,
	traits::{Block, Header, IdentifyAccount},
	AccountId32 as AccountId, MultiSignature, MultiSigner,
};
pub use sp_std::prelude::*;
pub use sp_version::RuntimeVersion;
use std::convert::{TryFrom, TryInto};
pub use transaction_payment::FeeDetails;
use transaction_payment::{InclusionFee, RuntimeDispatchInfo};

/// Api to talk with substrate-nodes
///
/// It is generic over the `RpcClient` trait, so you can use any rpc-backend you like.
///
/// # Custom Client Example
///
/// ```no_run
/// use substrate_api_client::rpc::json_req::author_submit_extrinsic;
/// use substrate_api_client::{
///     Api, ApiClientError, ApiResult, FromHexString, Hash, RpcInterface, XtStatus, PlainTipExtrinsicParams
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
///     ) -> Result<R, Box<dyn std::error::Error>> {
///         // you can figure this out...self.inner...send_json...
///         todo!()
///     }
/// }
///
/// impl RpcClient for MyClient {
///     fn get_request(&self, jsonreq: Value) -> ApiResult<String> {
///         self.send_json::<Value>("".into(), jsonreq)
///             .map(|v| v.to_string())
///             .map_err(|err| ApiClientError::RpcClient(err.to_string()))
///     }
///
///     fn send_extrinsic(
///         &self,
///         xthex_prefixed: String,
///         _exit_on: XtStatus,
///     ) -> ApiResult<Option<Hash>> {
///         let jsonreq = author_submit_extrinsic(&xthex_prefixed);
///         let res: String = self
///             .send_json("".into(), jsonreq)
///             .map_err(|err| ApiClientError::RpcClient(err.to_string()))?;
///         Ok(Some(Hash::from_hex(res)?))
///     }
/// }
///
/// let client = MyClient::new();
/// let _api = Api::<(), _, PlainTipExtrinsicParams>::new(client);
///
/// ```
#[derive(Clone)]
pub struct Api<Signer, Client, Params, Hash> {
	signer: Option<Signer>,
	genesis_hash: Hash,
	metadata: Metadata,
	runtime_version: RuntimeVersion,
	client: Client,
	extrinsic_params_builder: Option<Params>,
}

/// Public Api Interface.
impl<Signer, Client, Params> Api<Signer, Client, Params, Params::Hash>
where
	Signer: Pair,
	MultiSignature: From<Signer::Signature>,
	MultiSigner: From<Signer::Public>,
	Client: RpcInterface + RuntimeInterface,
	Params: ExtrinsicParams,
{
	/// Creates a new Api by retrieving data directly from the substrate node.
	pub fn new(client: Client) -> ApiResult<Self> {
		let genesis_hash = self.get_genesis_hash_from_node()?;
		info!("Got genesis hash: {:?}", genesis_hash);

		let metadata = client.get_metadata(&client).map(Metadata::try_from)??;
		debug!("Metadata: {:?}", metadata);

		let runtime_version = client.get_runtime_version(&client)?;
		info!("Runtime Version: {:?}", runtime_version);

		Ok(Self::new_offline(client, genesis_hash, metadata, runtime_version));
	}

	/// Creates a new Api, without any connection to the substrate node.
	pub fn new_offline(
		client: Client,
		genesis_hash: Params::Hash,
		metadata: Metadata,
		runtime_version: RuntimeVersion,
	) -> Self {
		Self {
			signer: None,
			genesis_hash,
			metadata,
			runtime_version,
			client,
			extrinsic_params_builder: None,
		}
	}

	#[must_use]
	pub fn set_signer(mut self, signer: Signer) -> Self {
		self.signer = Some(signer);
		self
	}

	pub fn set_extrinsic_params_builder(mut self, extrinsic_params: Params::OtherParams) -> Self {
		self.extrinsic_params_builder = Some(extrinsic_params);
		self
	}

	pub fn signer_account<AccountId: IdentifyAccount>(&self) -> Option<AccountId> {
		let pair = self.signer.as_ref()?;
		let multi_signer = MultiSigner::from(pair.public());
		Some(multi_signer.into_account())
	}

	pub fn extrinsic_params(&self, nonce: Params::Index) -> Params {
		let extrinsic_params_builder = self.extrinsic_params_builder.clone().unwrap_or_default();
		<Params as ExtrinsicParams>::new(
			self.runtime_version.spec_version,
			self.runtime_version.transaction_version,
			nonce,
			self.genesis_hash,
			extrinsic_params_builder,
		)
	}

	pub fn genesis_hash(&self) -> Params::Hash {
		self.genesis_hash
	}

	pub fn runtime_version(&self) -> RuntimeVersion {
		self.runtime_version
	}

	/// Get nonce of signer account from substrate node.
	pub fn get_nonce(&self) -> ApiResult<Params::Index> {
		if self.signer.is_none() {
			return Err(ApiClientError::NoSigner)
		}

		self.get_account_info(&self.signer_account().unwrap()).unwrap_or_default()
	}

	/// Private function, should only be called at start up.
	fn get_genesis_hash_from_node(&self) -> ApiResult<Params::Hash> {
		let jsonreq = json_req::chain_get_genesis_hash();
		let genesis = client.get_request(jsonreq)?;

		match genesis {
			Some(g) => Hash::from_hex(g).map_err(|e| e.into()),
			None => Err(ApiClientError::Genesis),
		}
	}
}

impl<Signer, Client, Params> RuntimeInterface for Api<Signer, Client, Params, Params::Hash>
where
	Signer: Pair,
	MultiSignature: From<Signer::Signature>,
	MultiSigner: From<Signer::Public>,
	Client: RpcInterface + RuntimeInterface,
	Params: ExtrinsicParams,
{
	fn get_metadata(&self) -> ApiResult<RuntimeMetadataPrefixed> {
		let jsonreq = json_req::state_get_metadata();
		let meta = client.get_request(jsonreq)?;

		if meta.is_none() {
			return Err(ApiClientError::MetadataFetch)
		}
		let metadata = Vec::from_hex(meta.unwrap())?;
		RuntimeMetadataPrefixed::decode(&mut metadata.as_slice()).map_err(|e| e.into())
	}

	fn get_runtime_version(&self) -> ApiResult<RuntimeVersion> {
		let jsonreq = json_req::state_get_runtime_version();
		let version = client.get_request(jsonreq)?;

		match version {
			Some(v) => serde_json::from_str(&v).map_err(|e| e.into()),
			None => Err(ApiClientError::RuntimeVersion),
		}
	}

	fn get_constant<C: Decode>(&self, pallet: &'static str, constant: &'static str)
		-> ApiResult<C>;
}

impl<Signer, Client, Params, Runtime> FrameSystemInterface<Runtime>
	for Api<Signer, Client, Params, Params::Hash>
where
	Signer: Pair,
	MultiSignature: From<Signer::Signature>,
	MultiSigner: From<Signer::Public>,
	Client: RpcInterface + RuntimeInterface,
	Params: ExtrinsicParams,
	Runtime: system::Config,
{
	fn get_account_info(
		&self,
		address: &Runtime::AccountId,
	) -> ApiResult<Option<AccountInfoFor<Runtime>>> {
		let storage_key: sp_core::storage::StorageKey = self
			.metadata
			.storage_map_key::<AccountId>("System", "Account", address.clone())?;

		info!("storage key is: 0x{}", hex::encode(&storage_key));
		self.get_storage_by_key_hash(storage_key, None)
	}

	fn get_account_data(
		&self,
		address: &Runtime::AccountId,
	) -> ApiResult<Option<Runtime::AccountData>> {
		self.get_account_info(address).map(|info| info.map(|i| i.data))
	}

	fn get_finalized_head(&self) -> ApiResult<Option<Runtime::Hash>> {
		let h = self.client.get_request(json_req::chain_get_finalized_head())?;
		match h {
			Some(hash) => Ok(Some(Hash::from_hex(hash)?)),
			None => Ok(None),
		}
	}

	fn get_header(&self, hash: Option<Runtime::Hash>) -> ApiResult<Option<Runtime::Header>> {
		let h = client.get_request(json_req::chain_get_header(hash))?;
		match h {
			Some(hash) => Ok(Some(serde_json::from_str(&hash)?)),
			None => Ok(None),
		}
	}

	fn get_block_hash(
		&self,
		number: Option<Runtime::BlockNumber>,
	) -> ApiResult<Option<Runtime::Hash>> {
		let h = self.client.get_request(json_req::chain_get_block_hash(number))?;
		match h {
			Some(hash) => Ok(Some(Hash::from_hex(hash)?)),
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
		let b = self.client.get_request(json_req::chain_get_block(hash))?;
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

impl<P, Client, Params, Runtime> FrameSystemInterface<Runtime> for Api<P, Client, Params>
where
	Client: RpcInterface,
	Params: ExtrinsicParams,
	Runtime: system::Config,
{
	pub fn get_request(&self, jsonreq: Value) -> ApiResult<Option<String>> {
		Self::_get_request(&self.client, jsonreq)
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
	) -> ApiResult<Option<rpc::ReadProof<Hash>>> {
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
	) -> ApiResult<Option<rpc::ReadProof<Hash>>> {
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
	) -> ApiResult<Option<rpc::ReadProof<Hash>>> {
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
	) -> ApiResult<Option<rpc::ReadProof<Hash>>> {
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
		self.client.send_extrinsic(xthex_prefixed, exit_on)
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
