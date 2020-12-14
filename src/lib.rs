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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use sp_std::prelude::*;

#[cfg(feature = "std")]
use std::sync::mpsc::Sender as ThreadOut;
#[cfg(feature = "std")]
use std::sync::mpsc::{channel, Receiver};

#[cfg(feature = "std")]
use std::convert::TryFrom;

use balances::AccountData as AccountDataGen;
use system::AccountInfo as AccountInfoGen;

#[cfg(feature = "std")]
use codec::{Decode, Encode};

#[cfg(feature = "std")]
use log::{debug, error, info};

#[cfg(feature = "std")]
use metadata::RuntimeMetadataPrefixed;
#[cfg(feature = "std")]
use sp_core::crypto::Pair;

#[cfg(feature = "std")]
use rpc::json_req;

#[cfg(feature = "std")]
use utils::*;

#[cfg(feature = "std")]
use sp_version::RuntimeVersion;

#[macro_use]
pub mod extrinsic;
#[cfg(feature = "std")]
pub mod events;
#[cfg(feature = "std")]
pub mod node_metadata;

#[cfg(feature = "std")]
use sp_core::storage::StorageKey;

#[cfg(feature = "std")]
use sc_rpc_api::state::ReadProof;

pub mod utils;

#[cfg(feature = "std")]
use utils::FromHexString;

#[cfg(feature = "std")]
pub mod rpc;

#[cfg(feature = "std")]
use events::{EventsDecoder, RawEvent, RuntimeEvent};
#[cfg(feature = "std")]
use sp_runtime::{generic::SignedBlock, AccountId32 as AccountId, MultiSignature};

pub extern crate sp_runtime;

pub use sp_core::H256 as Hash;

/// The block number type used in this runtime.
pub type BlockNumber = u64;
/// The timestamp moment type used in this runtime.
pub type Moment = u64;
/// Index of a transaction.
//fixme: make generic
pub type Index = u32;

// re-export useful types
pub use extrinsic::xt_primitives::{GenericAddress, GenericExtra, UncheckedExtrinsicV4};
#[cfg(feature = "std")]
pub use node_metadata::Metadata;

#[cfg(feature = "std")]
pub use rpc::XtStatus;

#[cfg(feature = "std")]
use sp_core::crypto::AccountId32;

#[cfg(feature = "std")]
use sp_runtime::{
    traits::{Block, Header},
    DeserializeOwned,
};

//fixme: make generic
pub type Balance = u128;

pub type AccountData = AccountDataGen<Balance>;
pub type AccountInfo = AccountInfoGen<Index, AccountData>;

#[cfg(feature = "std")]
type ApiResult<T> = Result<T, ApiClientError>;

#[cfg(feature = "std")]
#[derive(Clone)]
pub struct Api<P>
where
    P: Pair,
    MultiSignature: From<P::Signature>,
{
    pub url: String,
    pub signer: Option<P>,
    pub genesis_hash: Hash,
    pub metadata: Metadata,
    pub runtime_version: RuntimeVersion,
}

#[cfg(feature = "std")]
impl<P> Api<P>
where
    P: Pair,
    MultiSignature: From<P::Signature>,
{
    pub fn new(url: String) -> ApiResult<Self> {
        let genesis_hash = Self::_get_genesis_hash(url.clone())?;
        info!("Got genesis hash: {:?}", genesis_hash);

        let metadata = Self::_get_metadata(url.clone()).map(Metadata::try_from)??;
        debug!("Metadata: {:?}", metadata);

        let runtime_version = Self::_get_runtime_version(url.clone())?;
        info!("Runtime Version: {:?}", runtime_version);

        Ok(Self {
            url,
            signer: None,
            genesis_hash,
            metadata,
            runtime_version,
        })
    }

    pub fn set_signer(mut self, signer: P) -> Self {
        self.signer = Some(signer);
        self
    }

    pub fn signer_account(&self) -> Option<AccountId32> {
        let pair = self.signer.as_ref()?;
        let mut arr: [u8; 32] = Default::default();
        arr.clone_from_slice(pair.to_owned().public().as_ref());
        Some(AccountId32::from(arr))
    }

    fn _get_genesis_hash(url: String) -> ApiResult<Hash> {
        let jsonreq = json_req::chain_get_genesis_hash();
        let genesis = Self::_get_request(url, jsonreq.to_string())?;

        match genesis {
            Some(g) => Hash::from_hex(g).map_err(|e| e.into()),
            None => Err(ApiClientError::Genesis),
        }
    }

    fn _get_runtime_version(url: String) -> ApiResult<RuntimeVersion> {
        let jsonreq = json_req::state_get_runtime_version();
        let version = Self::_get_request(url, jsonreq.to_string())?;

        match version {
            Some(v) => serde_json::from_str(&v).map_err(|e| e.into()),
            None => Err(ApiClientError::RuntimeVersion),
        }
    }

    fn _get_metadata(url: String) -> ApiResult<RuntimeMetadataPrefixed> {
        let jsonreq = json_req::state_get_metadata();
        let meta = Self::_get_request(url, jsonreq.to_string())?;

        if meta.is_none() {
            return Err(ApiClientError::MetadataFetch);
        }
        let metadata = Vec::from_hex(meta.unwrap())?;
        RuntimeMetadataPrefixed::decode(&mut metadata.as_slice()).map_err(|e| e.into())
    }

    // low level access
    fn _get_request(url: String, jsonreq: String) -> ApiResult<Option<String>> {
        let (result_in, result_out) = channel();
        rpc::get(url, jsonreq, result_in)?;

        let str = result_out.recv()?;

        match &str[..] {
            "null" => Ok(None),
            _ => Ok(Some(str)),
        }
    }

    pub fn get_metadata(&self) -> ApiResult<RuntimeMetadataPrefixed> {
        Self::_get_metadata(self.url.clone())
    }

    pub fn get_spec_version(&self) -> ApiResult<u32> {
        Self::_get_runtime_version(self.url.clone()).map(|v| v.spec_version)
    }

    pub fn get_genesis_hash(&self) -> ApiResult<Hash> {
        Self::_get_genesis_hash(self.url.clone())
    }

    pub fn get_nonce(&self) -> ApiResult<u32> {
        if self.signer.is_none() {
            return Err(ApiClientError::NoSigner);
        }

        self.get_account_info(&self.signer_account().unwrap())
            .map(|acc_opt| acc_opt.map_or_else(|| 0, |acc| acc.nonce))
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
        let h = self.get_request(json_req::chain_get_finalized_head().to_string())?;
        match h {
            Some(hash) => Ok(Some(Hash::from_hex(hash)?)),
            None => Ok(None),
        }
    }

    pub fn get_header<H>(&self, hash: Option<Hash>) -> ApiResult<Option<H>>
    where
        H: Header + DeserializeOwned,
    {
        let h = self.get_request(json_req::chain_get_header(hash).to_string())?;
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
        let b = self.get_request(json_req::chain_get_block(hash).to_string())?;
        match b {
            Some(block) => Ok(Some(serde_json::from_str(&block)?)),
            None => Ok(None),
        }
    }

    pub fn get_request(&self, jsonreq: String) -> ApiResult<Option<String>> {
        Self::_get_request(self.url.clone(), jsonreq)
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
        let s = self.get_request(jsonreq.to_string())?;

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
    ) -> ApiResult<Option<ReadProof<Hash>>> {
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
    ) -> ApiResult<Option<ReadProof<Hash>>> {
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
    ) -> ApiResult<Option<ReadProof<Hash>>> {
        let jsonreq = json_req::state_get_read_proof(keys, at_block);
        let p = self.get_request(jsonreq.to_string())?;
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
        let k = self.get_request(jsonreq.to_string())?;
        match k {
            Some(keys) => Ok(Some(serde_json::from_str(&keys)?)),
            None => Ok(None),
        }
    }

    pub fn send_extrinsic(
        &self,
        xthex_prefixed: String,
        exit_on: XtStatus,
    ) -> ApiResult<Option<Hash>> {
        debug!("sending extrinsic: {:?}", xthex_prefixed);

        let jsonreq = json_req::author_submit_and_watch_extrinsic(&xthex_prefixed).to_string();

        let (result_in, result_out) = channel();
        match exit_on {
            XtStatus::Finalized => {
                rpc::send_extrinsic_and_wait_until_finalized(self.url.clone(), jsonreq, result_in)?;
                let res = result_out.recv()?;
                info!("finalized: {}", res);
                Ok(Some(Hash::from_hex(res)?))
            }
            XtStatus::InBlock => {
                rpc::send_extrinsic_and_wait_until_in_block(self.url.clone(), jsonreq, result_in)?;
                let res = result_out.recv()?;
                info!("inBlock: {}", res);
                Ok(Some(Hash::from_hex(res)?))
            }
            XtStatus::Broadcast => {
                rpc::send_extrinsic_and_wait_until_broadcast(self.url.clone(), jsonreq, result_in)?;
                let res = result_out.recv()?;
                info!("broadcast: {}", res);
                Ok(None)
            }
            XtStatus::Ready => {
                rpc::send_extrinsic(self.url.clone(), jsonreq, result_in)?;
                let res = result_out.recv()?;
                info!("ready: {}", res);
                Ok(None)
            }
            _ => Err(ApiClientError::UnsupportedXtStatus(exit_on)),
        }
    }

    pub fn subscribe_events(&self, sender: ThreadOut<String>) -> ApiResult<()> {
        debug!("subscribing to events");
        let key = storage_key("System", "Events");
        let jsonreq = json_req::state_subscribe_storage(vec![key]).to_string();
        rpc::start_subscriber(self.url.clone(), jsonreq, sender).map_err(|e| e.into())
    }

    pub fn subscribe_finalized_heads(&self, sender: ThreadOut<String>) -> ApiResult<()> {
        debug!("subscribing to finalized heads");
        let jsonreq = json_req::chain_subscribe_finalized_heads().to_string();
        rpc::start_subscriber(self.url.clone(), jsonreq, sender).map_err(|e| e.into())
    }

    pub fn wait_for_event<E: Decode>(
        &self,
        module: &str,
        variant: &str,
        decoder: Option<EventsDecoder>,
        receiver: &Receiver<String>,
    ) -> ApiResult<E> {
        let raw = self.wait_for_raw_event(module, variant, decoder, receiver)?;
        E::decode(&mut &raw.data[..]).map_err(|e| e.into())
    }

    pub fn wait_for_raw_event(
        &self,
        module: &str,
        variant: &str,
        decoder: Option<EventsDecoder>,
        receiver: &Receiver<String>,
    ) -> ApiResult<RawEvent> {
        let event_decoder = match decoder {
            Some(d) => d,
            None => EventsDecoder::try_from(self.metadata.clone())?,
        };

        loop {
            let event_str = receiver.recv()?;
            let _events = event_decoder.decode_events(&mut Vec::from_hex(event_str)?.as_slice());
            info!("wait for raw event");
            match _events {
                Ok(raw_events) => {
                    for (phase, event) in raw_events.into_iter() {
                        info!("Decoded Event: {:?}, {:?}", phase, event);
                        match event {
                            RuntimeEvent::Raw(raw)
                                if raw.module == module && raw.variant == variant =>
                            {
                                return Ok(raw);
                            }
                            _ => debug!("ignoring unsupported module event: {:?}", event),
                        }
                    }
                }
                Err(_) => error!("couldn't decode event record list"),
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[cfg(feature = "std")]
pub enum ApiClientError {
    #[error("Fetching genesis hash failed. Are you connected to the correct endpoint?")]
    Genesis,
    #[error("Fetching runtime version failed. Are you connected to the correct endpoint?")]
    RuntimeVersion,
    #[error("Fetching Metadata failed. Are you connected to the correct endpoint?")]
    MetadataFetch,
    #[error("Operation needs a signer to be set in the api")]
    NoSigner,
    #[error("WebSocket Error: {0}")]
    WebSocket(#[from] ws::Error),
    #[error("ChannelReceiveError, sender is disconnected: {0}")]
    Disconnected(#[from] sp_std::sync::mpsc::RecvError),
    #[error("Metadata Error: {0}")]
    Metadata(#[from] crate::node_metadata::MetadataError),
    #[error("Events Error: {0}")]
    Events(#[from] crate::events::EventsError),
    #[error("Error decoding storage value: {0}")]
    StorageValueDecode(#[from] extrinsic::codec::Error),
    #[error("Received invalid hex string: {0}")]
    InvalidHexString(#[from] hex::FromHexError),
    #[error("Error deserializing with serde: {0}")]
    Deserializing(#[from] serde_json::Error),
    #[error("UnsupportedXtStatus Error: Can only wait for finalized, in block, broadcast and ready. Waited for: {0:?}")]
    UnsupportedXtStatus(XtStatus),
}
