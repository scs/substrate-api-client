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
	api::error::{Error, Result},
	rpc::Request,
	runtime_api::RuntimeApiClient,
	GetAccountInformation,
};
use ac_compose_macros::rpc_params;
use ac_node_api::metadata::Metadata;
use ac_primitives::{Config, ExtrinsicParams, SignExtrinsic};
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use alloc::sync::Arc;
use codec::Decode;
use core::convert::TryFrom;
use frame_metadata::RuntimeMetadataPrefixed;
use log::{debug, info};
use sp_core::Bytes;
use sp_version::RuntimeVersion;

/// Api to talk with substrate-nodes
///
/// It is generic over the `Request` trait, so you can use any rpc-backend you like.
#[derive(Clone)]
pub struct Api<T: Config, Client> {
	signer: Option<T::ExtrinsicSigner>,
	genesis_hash: T::Hash,
	metadata: Metadata,
	runtime_version: RuntimeVersion,
	client: Arc<Client>,
	runtime_api: RuntimeApiClient<T, Client>,
	additional_extrinsic_params:
		Option<<T::ExtrinsicParams as ExtrinsicParams<T::Index, T::Hash>>::AdditionalParams>,
}

impl<T: Config, Client> Api<T, Client> {
	/// Create a new api instance without any node interaction.
	pub fn new_offline(
		genesis_hash: T::Hash,
		metadata: Metadata,
		runtime_version: RuntimeVersion,
		client: Client,
	) -> Self {
		let client = Arc::new(client);
		let runtime_api = RuntimeApiClient::new(client.clone());
		Self {
			signer: None,
			genesis_hash,
			metadata,
			runtime_version,
			client,
			runtime_api,
			additional_extrinsic_params: None,
		}
	}

	/// Set the api signer account.
	pub fn set_signer(&mut self, signer: T::ExtrinsicSigner) {
		self.signer = Some(signer);
	}

	/// Get the private key pair of the api signer.
	pub fn signer(&self) -> Option<&T::ExtrinsicSigner> {
		self.signer.as_ref()
	}

	/// Get the cached genesis hash of the substrate node.
	pub fn genesis_hash(&self) -> T::Hash {
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

	/// Set the additional params.
	pub fn set_additional_params(
		&mut self,
		add_params: <T::ExtrinsicParams as ExtrinsicParams<T::Index, T::Hash>>::AdditionalParams,
	) {
		self.additional_extrinsic_params = Some(add_params);
	}

	/// Access the RuntimeApi.
	pub fn runtime_api(&self) -> &RuntimeApiClient<T, Client> {
		&self.runtime_api
	}

	/// Get the extrinsic params with the set additional params. If no additional params are set,
	/// the default is taken.
	pub fn extrinsic_params(&self, nonce: T::Index) -> T::ExtrinsicParams {
		let additional_extrinsic_params =
			self.additional_extrinsic_params.clone().unwrap_or_default();
		T::ExtrinsicParams::new(
			self.runtime_version.spec_version,
			self.runtime_version.transaction_version,
			nonce,
			self.genesis_hash,
			additional_extrinsic_params,
		)
	}
}

impl<T, Client> Api<T, Client>
where
	T: Config,
	Client: Request,
{
	/// Create a new Api client with call to the node to retrieve metadata.
	#[maybe_async::async_impl]
	pub async fn new(client: Client) -> Result<Self> {
		let genesis_hash_future = Self::get_genesis_hash(&client);
		let metadata_future = Self::get_metadata(&client);
		let runtime_version_future = Self::get_runtime_version(&client);

		let (genesis_hash, metadata, runtime_version) = futures_util::future::try_join3(
			genesis_hash_future,
			metadata_future,
			runtime_version_future,
		)
		.await?;
		info!("Got genesis hash: {:?}", genesis_hash);
		debug!("Metadata: {:?}", metadata);
		info!("Runtime Version: {:?}", runtime_version);
		Ok(Self::new_offline(genesis_hash, metadata, runtime_version, client))
	}

	/// Create a new Api client with call to the node to retrieve metadata.
	#[maybe_async::sync_impl]
	pub fn new(client: Client) -> Result<Self> {
		let genesis_hash = Self::get_genesis_hash(&client)?;
		info!("Got genesis hash: {:?}", genesis_hash);

		let metadata = Self::get_metadata(&client)?;
		debug!("Metadata: {:?}", metadata);

		let runtime_version = Self::get_runtime_version(&client)?;
		info!("Runtime Version: {:?}", runtime_version);

		Ok(Self::new_offline(genesis_hash, metadata, runtime_version, client))
	}
}

#[maybe_async::maybe_async(?Send)]
pub trait UpdateRuntime {
	/// Updates the runtime and metadata of the api via node query.
	/// Ideally, this function is called if a substrate update runtime event is encountered.
	async fn update_runtime(&mut self) -> Result<()>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> UpdateRuntime for Api<T, Client>
where
	T: Config,
	Client: Request,
{
	#[maybe_async::sync_impl]
	fn update_runtime(&mut self) -> Result<()> {
		let metadata = Self::get_metadata(&self.client)?;
		let runtime_version = Self::get_runtime_version(&self.client)?;

		debug!("Metadata: {:?}", metadata);
		info!("Runtime Version: {:?}", runtime_version);

		self.metadata = metadata;
		self.runtime_version = runtime_version;
		Ok(())
	}

	#[maybe_async::async_impl(?Send)]
	async fn update_runtime(&mut self) -> Result<()> {
		let metadata_future = Self::get_metadata(&self.client);
		let runtime_version_future = Self::get_runtime_version(&self.client);

		let (metadata, runtime_version) =
			futures_util::future::try_join(metadata_future, runtime_version_future).await?;

		debug!("Metadata: {:?}", metadata);
		info!("Runtime Version: {:?}", runtime_version);

		self.metadata = metadata;
		self.runtime_version = runtime_version;
		Ok(())
	}
}

impl<T, Client> Api<T, Client>
where
	T: Config,
	Client: Request,
{
	/// Get the public part of the api signer account.
	pub fn signer_account(&self) -> Option<&T::AccountId> {
		let pair = self.signer.as_ref()?;
		Some(pair.public_account_id())
	}

	/// Get nonce of self signer account.
	#[maybe_async::maybe_async(?Send)]
	pub async fn get_nonce(&self) -> Result<T::Index> {
		let account = self.signer_account().ok_or(Error::NoSigner)?;
		self.get_account_nonce(account).await
	}
}

/// Private node query methods. They should be used internally only, because the user should retrieve the data from the struct cache.
/// If an up-to-date query is necessary, cache should be updated beforehand.
impl<T, Client> Api<T, Client>
where
	T: Config,
	Client: Request,
{
	/// Get the genesis hash from node via websocket query.
	#[maybe_async::maybe_async(?Send)]
	async fn get_genesis_hash(client: &Client) -> Result<T::Hash> {
		let genesis: Option<T::Hash> =
			client.request("chain_getBlockHash", rpc_params![Some(0)]).await?;
		genesis.ok_or(Error::FetchGenesisHash)
	}

	/// Get runtime version from node via websocket query.
	#[maybe_async::maybe_async(?Send)]
	async fn get_runtime_version(client: &Client) -> Result<RuntimeVersion> {
		let version: RuntimeVersion =
			client.request("state_getRuntimeVersion", rpc_params![]).await?;
		Ok(version)
	}

	/// Get metadata from node via websocket query.
	#[maybe_async::maybe_async(?Send)]
	async fn get_metadata(client: &Client) -> Result<Metadata> {
		let metadata_bytes: Bytes = client.request("state_getMetadata", rpc_params![]).await?;

		let metadata = RuntimeMetadataPrefixed::decode(&mut metadata_bytes.0.as_slice())?;
		Metadata::try_from(metadata).map_err(|e| e.into())
	}
}
#[cfg(test)]
mod tests {
	use super::*;
	use crate::rpc::mocks::RpcClientMock;
	use ac_primitives::{
		AssetRuntimeConfig, DefaultRuntimeConfig, GenericAdditionalParams, GenericExtrinsicParams,
		PlainTip,
	};
	use frame_metadata::{v14::ExtrinsicMetadata, RuntimeMetadata};
	use scale_info::form::PortableForm;
	use sp_core::H256;
	use std::{collections::HashMap, fs};

	fn create_mock_api(
		genesis_hash: H256,
		runtime_version: RuntimeVersion,
		metadata: Metadata,
		data: HashMap<String, String>,
	) -> Api<DefaultRuntimeConfig, RpcClientMock> {
		let client = RpcClientMock::new(data);
		Api::new_offline(genesis_hash, metadata, runtime_version, client)
	}

	#[test]
	fn api_extrinsic_params_works() {
		// Create new api.
		let genesis_hash = H256::random();
		let runtime_version = RuntimeVersion::default();
		let encoded_metadata = fs::read("./ksm_metadata_v14.bin").unwrap();
		let metadata: RuntimeMetadataPrefixed =
			Decode::decode(&mut encoded_metadata.as_slice()).unwrap();
		let metadata = Metadata::try_from(metadata).unwrap();

		let mut api =
			create_mock_api(genesis_hash, runtime_version.clone(), metadata, Default::default());

		// Information for Era for mortal transactions.
		let additional_params = GenericAdditionalParams::new();
		api.set_additional_params(additional_params);

		let nonce = 6;
		let retrieved_params = api.extrinsic_params(nonce);

		let expected_params = GenericExtrinsicParams::<AssetRuntimeConfig, PlainTip<u128>>::new(
			runtime_version.spec_version,
			runtime_version.transaction_version,
			nonce,
			genesis_hash,
			Default::default(),
		);

		assert_eq!(expected_params, retrieved_params)
	}

	#[test]
	fn api_runtime_update_works() {
		let runtime_version = RuntimeVersion { spec_version: 10, ..Default::default() };
		// Update metadata
		let encoded_metadata: Bytes = fs::read("./ksm_metadata_v14.bin").unwrap().into();
		let runtime_metadata_prefixed: RuntimeMetadataPrefixed =
			Decode::decode(&mut encoded_metadata.0.as_slice()).unwrap();
		let mut runtime_metadata = match runtime_metadata_prefixed.1 {
			RuntimeMetadata::V14(ref metadata) => metadata.clone(),
			_ => unimplemented!(),
		};

		let metadata = Metadata::try_from(runtime_metadata_prefixed).unwrap();

		runtime_metadata.extrinsic = ExtrinsicMetadata::<PortableForm> {
			ty: runtime_metadata.extrinsic.ty,
			version: 0,
			signed_extensions: Vec::new(),
		};
		let changed_runtime_metadata_prefixed =
			RuntimeMetadataPrefixed(1635018093, RuntimeMetadata::V14(runtime_metadata));
		let changed_metadata = Metadata::try_from(changed_runtime_metadata_prefixed).unwrap();

		let data = HashMap::<String, String>::from([
			(
				"chain_getBlockHash".to_owned(),
				serde_json::to_string(&Some(H256::from([1u8; 32]))).unwrap(),
			),
			(
				"state_getRuntimeVersion".to_owned(),
				serde_json::to_string(&runtime_version).unwrap(),
			),
			("state_getMetadata".to_owned(), serde_json::to_string(&encoded_metadata).unwrap()),
		]);
		let mut api =
			create_mock_api(Default::default(), Default::default(), changed_metadata, data);

		// Ensure current metadata and runtime version are different.
		assert_ne!(api.metadata.extrinsic(), metadata.extrinsic());
		assert_ne!(api.runtime_version, runtime_version);

		// Update runtime.
		api.update_runtime().unwrap();

		// Ensure metadata and runtime version have been updated.
		assert_eq!(api.metadata.extrinsic(), metadata.extrinsic());
		assert_eq!(api.runtime_version, runtime_version);
	}
}
