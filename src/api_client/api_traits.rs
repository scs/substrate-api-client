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

/// Interface to runtime specified variables.
pub trait RuntimeInterface {
	fn get_metadata(&self) -> ApiResult<RuntimeMetadataPrefixed>;

	fn get_runtime_version(&self) -> ApiResult<RuntimeVersion>;

	fn get_constant<C: Decode>(&self, pallet: &'static str, constant: &'static str)
		-> ApiResult<C>;
}

/// Interface to the substrate rpc interface.
pub trait RpcInterface {
	/// Sends a RPC request that returns a String.
	fn get_request(&self, jsonreq: serde_json::Value) -> ApiResult<Option<String>>;

	/// Send an extrinsic, but returns immediately. It does not wait for any status responses.
	fn send_extrinsic(&self, xthex_prefixed: String, exit_on: XtStatus) -> ApiResult<Option<Hash>>;

	/// Send a hex encoded extrinsic and optionally returns the block hash the extrinsic is included in.
	/// Provided it is watched until InBlock or Finalized. Otherwise a None is returned.
	fn submit_and_watch_extrinsic(
		&self,
		xthex_prefixed: String,
		exit_on: XtStatus,
	) -> ApiResult<Option<Hash>>;
}

/// Interface to common frame system pallet information.
pub trait FrameSystemInterface<Runtime: system::Config> {
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

/// Generic interface to substrate storage.
pub trait GenericStorageInterface<Hash> {
	fn get_storage_value<V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Hash>,
	) -> ApiResult<Option<V>>;

	fn get_storage_map<K: Encode, V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		map_key: K,
		at_block: Option<Hash>,
	) -> ApiResult<Option<V>>;

	fn get_storage_map_key_prefix(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
	) -> ApiResult<StorageKey>;

	fn get_storage_double_map<K: Encode, Q: Encode, V: Decode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
		second: Q,
		at_block: Option<Hash>,
	) -> ApiResult<Option<V>>;

	fn get_storage_by_key_hash<V: Decode>(
		&self,
		key: StorageKey,
		at_block: Option<Hash>,
	) -> ApiResult<Option<V>>;

	fn get_opaque_storage_by_key_hash(
		&self,
		key: StorageKey,
		at_block: Option<Hash>,
	) -> ApiResult<Option<Vec<u8>>>;

	fn get_storage_value_proof(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		at_block: Option<Hash>,
	) -> ApiResult<Option<ReadProof<Hash>>>;

	fn get_storage_map_proof<K: Encode, V: Decode + Clone>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		map_key: K,
		at_block: Option<Hash>,
	) -> ApiResult<Option<ReadProof<Hash>>>;

	fn get_storage_double_map_proof<K: Encode, Q: Encode, V: Decode + Clone>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
		second: Q,
		at_block: Option<Hash>,
	) -> ApiResult<Option<ReadProof<Hash>>>;

	fn get_storage_proof_by_keys(
		&self,
		keys: Vec<StorageKey>,
		at_block: Option<Hash>,
	) -> ApiResult<Option<ReadProof<Hash>>>;

	fn get_keys(&self, key: StorageKey, at_block: Option<Hash>) -> ApiResult<Option<Vec<String>>>;
}

/// Interface to common calls of the substrate balances pallet.
pub trait BalanceInterface<Runtime: balances::Config + system::Config> {
	fn get_fee_details(
		&self,
		xthex_prefixed: &str,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<FeeDetails<Runtime::Balance>>>;

	fn get_payment_info(
		&self,
		xthex_prefixed: &str,
		at_block: Option<Runtime::Hash>,
	) -> ApiResult<Option<RuntimeDispatchInfo<Runtime::Balance>>>;

	fn get_existential_deposit(&self) -> ApiResult<Runtime::Balance>;
}
