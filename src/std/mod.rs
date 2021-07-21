pub use metadata::RuntimeMetadataPrefixed;
pub use serde_json::Value;
pub use sp_core::crypto::Pair;
pub use sp_core::storage::StorageKey;
pub use sp_runtime::traits::{Block, Header};
pub use sp_runtime::{
    generic::SignedBlock, traits::IdentifyAccount, AccountId32 as AccountId, MultiSignature,
    MultiSigner,
};
pub use sp_std::prelude::*;
pub use sp_version::RuntimeVersion;

pub use crate::std::rpc::XtStatus;
pub use crate::utils::FromHexString;
pub use node_metadata::Metadata;

pub mod rpc;

mod node_metadata;

use std::convert::TryFrom;

use codec::{Decode, Encode};
use log::{debug, info};
use serde::de::DeserializeOwned;

use crate::extrinsic;
use crate::rpc::json_req;
use crate::{AccountData, AccountInfo, Hash};

pub type ApiResult<T> = Result<T, ApiClientError>;

pub trait RpcClient {
    /// Sends a RPC request that returns a String
    fn get_request(&self, jsonreq: serde_json::Value) -> ApiResult<String>;

    /// Send a RPC request that returns a SHA256 hash
    fn send_extrinsic(&self, xthex_prefixed: String, exit_on: XtStatus) -> ApiResult<Option<Hash>>;
}

#[derive(Clone)]
pub struct Api<P, Client>
where
    Client: RpcClient,
{
    pub signer: Option<P>,
    pub genesis_hash: Hash,
    pub metadata: Metadata,
    pub runtime_version: RuntimeVersion,
    client: Client,
}

impl<P, Client> Api<P, Client>
where
    P: Pair,
    MultiSignature: From<P::Signature>,
    MultiSigner: From<P::Public>,
    Client: RpcClient,
{
    pub fn signer_account(&self) -> Option<AccountId> {
        let pair = self.signer.as_ref()?;
        let multi_signer = MultiSigner::from(pair.public());
        Some(multi_signer.into_account())
    }

    pub fn get_nonce(&self) -> ApiResult<u32> {
        if self.signer.is_none() {
            return Err(ApiClientError::NoSigner);
        }

        self.get_account_info(&self.signer_account().unwrap())
            .map(|acc_opt| acc_opt.map_or_else(|| 0, |acc| acc.nonce))
    }
}

impl<P, Client> Api<P, Client>
where
    Client: RpcClient,
{
    pub fn new(client: Client) -> ApiResult<Self> {
        let genesis_hash = Self::_get_genesis_hash(&client)?;
        info!("Got genesis hash: {:?}", genesis_hash);

        let metadata = Self::_get_metadata(&client).map(Metadata::try_from)??;
        debug!("Metadata: {:?}", metadata);

        let runtime_version = Self::_get_runtime_version(&client)?;
        info!("Runtime Version: {:?}", runtime_version);

        Ok(Self {
            signer: None,
            genesis_hash,
            metadata,
            runtime_version,
            client,
        })
    }

    pub fn set_signer(mut self, signer: P) -> Self {
        self.signer = Some(signer);
        self
    }

    fn _get_genesis_hash(client: &Client) -> ApiResult<Hash> {
        let jsonreq = json_req::chain_get_genesis_hash();
        let genesis = Self::_get_request(client, jsonreq)?;

        match genesis {
            Some(g) => Hash::from_hex(g).map_err(|e| e.into()),
            None => Err(ApiClientError::Genesis),
        }
    }

    fn _get_runtime_version(client: &Client) -> ApiResult<RuntimeVersion> {
        let jsonreq = json_req::state_get_runtime_version();
        let version = Self::_get_request(client, jsonreq)?;

        match version {
            Some(v) => serde_json::from_str(&v).map_err(|e| e.into()),
            None => Err(ApiClientError::RuntimeVersion),
        }
    }

    fn _get_metadata(client: &Client) -> ApiResult<RuntimeMetadataPrefixed> {
        let jsonreq = json_req::state_get_metadata();
        let meta = Self::_get_request(client, jsonreq)?;

        if meta.is_none() {
            return Err(ApiClientError::MetadataFetch);
        }
        let metadata = Vec::from_hex(meta.unwrap())?;
        RuntimeMetadataPrefixed::decode(&mut metadata.as_slice()).map_err(|e| e.into())
    }

    // low level access
    fn _get_request(client: &Client, jsonreq: Value) -> ApiResult<Option<String>> {
        let str = client.get_request(jsonreq)?;

        match &str[..] {
            "null" => Ok(None),
            _ => Ok(Some(str)),
        }
    }

    pub fn get_metadata(&self) -> ApiResult<RuntimeMetadataPrefixed> {
        Self::_get_metadata(&self.client)
    }

    pub fn get_spec_version(&self) -> ApiResult<u32> {
        Self::_get_runtime_version(&self.client).map(|v| v.spec_version)
    }

    pub fn get_genesis_hash(&self) -> ApiResult<Hash> {
        Self::_get_genesis_hash(&self.client)
    }

    pub fn get_account_info(&self, address: &AccountId) -> ApiResult<Option<AccountInfo>> {
        let storagekey: sp_core::storage::StorageKey = self
            .metadata
            .storage_map_key::<AccountId, AccountInfo>("System", "Account", address.clone())?;
        info!("storagekey {:?}", storagekey);
        info!("storage key is: 0x{}", hex::encode(storagekey.0.clone()));
        self.get_storage_by_key_hash(storagekey, None)
    }

    pub fn get_account_data(&self, address: &AccountId) -> ApiResult<Option<AccountData>> {
        self.get_account_info(address)
            .map(|info| info.map(|i| i.data))
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

    pub fn get_block<B>(&self, hash: Option<Hash>) -> ApiResult<Option<B>>
    where
        B: Block + DeserializeOwned,
    {
        Self::get_signed_block(self, hash).map(|sb_opt| sb_opt.map(|sb| sb.block))
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

    pub fn get_request(&self, jsonreq: Value) -> ApiResult<Option<String>> {
        Self::_get_request(&self.client, jsonreq)
    }

    pub fn get_storage_value<V: Decode>(
        &self,
        storage_prefix: &'static str,
        storage_key_name: &'static str,
        at_block: Option<Hash>,
    ) -> ApiResult<Option<V>> {
        let storagekey = self
            .metadata
            .storage_value_key(storage_prefix, storage_key_name)?;
        info!("storage key is: 0x{}", hex::encode(storagekey.0.clone()));
        self.get_storage_by_key_hash(storagekey, at_block)
    }

    pub fn get_storage_map<K: Encode, V: Decode + Clone>(
        &self,
        storage_prefix: &'static str,
        storage_key_name: &'static str,
        map_key: K,
        at_block: Option<Hash>,
    ) -> ApiResult<Option<V>> {
        let storagekey =
            self.metadata
                .storage_map_key::<K, V>(storage_prefix, storage_key_name, map_key)?;
        info!("storage key is: 0x{}", hex::encode(storagekey.0.clone()));
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

    pub fn get_storage_double_map<K: Encode, Q: Encode, V: Decode + Clone>(
        &self,
        storage_prefix: &'static str,
        storage_key_name: &'static str,
        first: K,
        second: Q,
        at_block: Option<Hash>,
    ) -> ApiResult<Option<V>> {
        let storagekey = self.metadata.storage_double_map_key::<K, Q, V>(
            storage_prefix,
            storage_key_name,
            first,
            second,
        )?;
        info!("storage key is: 0x{}", hex::encode(storagekey.0.clone()));
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
        let storagekey = self
            .metadata
            .storage_value_key(storage_prefix, storage_key_name)?;
        info!("storage key is: 0x{}", hex::encode(storagekey.0.clone()));
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
            self.metadata
                .storage_map_key::<K, V>(storage_prefix, storage_key_name, map_key)?;
        info!("storage key is: 0x{}", hex::encode(storagekey.0.clone()));
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
        let storagekey = self.metadata.storage_double_map_key::<K, Q, V>(
            storage_prefix,
            storage_key_name,
            first,
            second,
        )?;
        info!("storage key is: 0x{}", hex::encode(storagekey.0.clone()));
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
        self.client
            .send_extrinsic(xthex_prefixed, XtStatus::Broadcast)
    }
}

#[derive(Debug, thiserror::Error)]

pub enum ApiClientError {
    #[error("Fetching genesis hash failed. Are you connected to the correct endpoint?")]
    Genesis,
    #[error("Fetching runtime version failed. Are you connected to the correct endpoint?")]
    RuntimeVersion,
    #[error("Fetching Metadata failed. Are you connected to the correct endpoint?")]
    MetadataFetch,
    #[error("Operation needs a signer to be set in the api")]
    NoSigner,
    #[cfg(feature = "ws-client")]
    #[error("WebSocket Error: {0}")]
    WebSocket(#[from] ws::Error),
    #[error("RpcClient error: {0}")]
    RpcClient(String),
    #[error("ChannelReceiveError, sender is disconnected: {0}")]
    Disconnected(#[from] sp_std::sync::mpsc::RecvError),
    #[error("Metadata Error: {0}")]
    Metadata(#[from] node_metadata::MetadataError),
    #[cfg(feature = "ws-client")]
    #[error("Events Error: {0}")]
    Events(#[from] rpc::EventsError),
    #[error("Error decoding storage value: {0}")]
    StorageValueDecode(#[from] extrinsic::codec::Error),
    #[error("Received invalid hex string: {0}")]
    InvalidHexString(#[from] hex::FromHexError),
    #[error("Error deserializing with serde: {0}")]
    Deserializing(#[from] serde_json::Error),
    #[error("UnsupportedXtStatus Error: Can only wait for finalized, in block, broadcast and ready. Waited for: {0:?}")]
    UnsupportedXtStatus(XtStatus),
}
