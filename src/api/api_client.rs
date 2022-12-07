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
	generic::SignedBlock,
	traits::{Block, Header, IdentifyAccount},
	AccountId32, MultiSignature, MultiSigner,
};
pub use sp_std::prelude::*;

use crate::{
	api::interfaces::frame_system::GetAccountInformation,
	rpc::{json_req, RpcClient},
};
use ac_node_api::metadata::Metadata;
use ac_primitives::{ExtrinsicParams, FrameSystemConfig};
use codec::Decode;
use core::convert::TryFrom;
use log::{debug, info};
use sp_version::RuntimeVersion;

/// Api to talk with substrate-nodes
///
/// It is generic over the `RpcClient` trait, so you can use any rpc-backend you like.
///
/// # Custom Client Example
///
/// ```no_run
/// use substrate_api_client::rpc::json_req::author_submit_extrinsic;
/// use substrate_api_client::{
///     Api, ApiClientError, ApiResult, FromHexString,  RpcClient, rpc::Error as RpcClientError,  XtStatus, PlainTipExtrinsicParams, rpc::Result as RpcResult
/// };
/// use serde_json::Value;
/// use kitchensink_runtime::Runtime;
///
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
///     fn send_extrinsic<Hash: FromHexString>(
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
/// let _api = Api::<(), _, PlainTipExtrinsicParams<Runtime>, Runtime>::new(client);
///
/// ```
#[derive(Clone)]
pub struct Api<Signer, Client, Params, Runtime>
where
	Runtime: FrameSystemConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
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
	Runtime: FrameSystemConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
{
	/// Create a new api instance without any node interaction.
	pub fn new_offline(
		genesis_hash: Runtime::Hash,
		metadata: Metadata,
		runtime_version: RuntimeVersion,
		client: Client,
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

	/// Set the api signer account.
	pub fn set_signer(&mut self, signer: Signer) {
		self.signer = Some(signer);
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
	Client: RpcClient,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
	Runtime::Hash: FromHexString,
{
	pub fn new(client: Client) -> ApiResult<Self> {
		let genesis_hash = Self::get_genesis_hash(&client)?;
		info!("Got genesis hash: {:?}", genesis_hash);

		let metadata = Self::get_metadata(&client).map(Metadata::try_from)??;
		debug!("Metadata: {:?}", metadata);

		let runtime_version = Self::get_runtime_version(&client)?;
		info!("Runtime Version: {:?}", runtime_version);

		Ok(Self::new_offline(genesis_hash, metadata, runtime_version, client))
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

	#[cfg(feature = "ws-client")]
	pub fn send_extrinsic(
		&self,
		xthex_prefixed: String,
		exit_on: XtStatus,
	) -> ApiResult<Option<Runtime::Hash>> {
		debug!("sending extrinsic: {:?}", xthex_prefixed);
		self.client
			.send_extrinsic(xthex_prefixed, exit_on)
			.map_err(ApiClientError::RpcClient)
	}

	#[cfg(not(feature = "ws-client"))]
	pub fn send_extrinsic(&self, xthex_prefixed: String) -> ApiResult<Option<Runtime::Hash>> {
		debug!("sending extrinsic: {:?}", xthex_prefixed);
		// XtStatus should never be used used but we need to put something
		self.client.send_extrinsic(xthex_prefixed, XtStatus::Broadcast)
	}
}

impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Signer: Pair,
	MultiSignature: From<Signer::Signature>,
	Client: RpcClient,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
	Runtime::AccountId: From<Signer::Public>,
{
	/// Get the public part of the api signer account.
	pub fn signer_account(&self) -> Option<Runtime::AccountId> {
		let pair = self.signer.as_ref()?;
		Some(pair.public().into())
	}

	/// Get nonce of self signer account.
	pub fn get_nonce(&self) -> ApiResult<Runtime::Index> {
		let account = self.signer_account().ok_or(ApiClientError::NoSigner)?;
		self.get_account_info(&account)
			.map(|acc_opt| acc_opt.map_or_else(|| 0u32.into(), |acc| acc.nonce))
	}
}

/// Private node query methods. They should be used internally only, because the user should retrieve the data from the struct cache.
/// If an up-to-date query is necessary, cache should be updated beforehand.
impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Client: RpcClient,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
	Runtime::Hash: FromHexString,
{
	fn get_genesis_hash(client: &Client) -> ApiResult<Runtime::Hash> {
		let jsonreq = json_req::chain_get_genesis_hash();
		let genesis = client.get_request(jsonreq)?;

		match genesis {
			Some(g) => Runtime::Hash::from_hex(g).map_err(|e| e.into()),
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
