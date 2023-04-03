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
	GetAccountInformation,
};
use ac_compose_macros::rpc_params;
use ac_node_api::metadata::Metadata;
use ac_primitives::{Bytes, ExtrinsicParams, FrameSystemConfig, RuntimeVersion, SignExtrinsic};
use codec::Decode;
use core::convert::TryFrom;
use frame_metadata::RuntimeMetadataPrefixed;
use log::{debug, info};

/// Api to talk with substrate-nodes
///
/// It is generic over the `Request` trait, so you can use any rpc-backend you like.
///
/// # Custom Client Example
///
/// ```no_run
/// use substrate_api_client::{
///     Api, rpc::Request, rpc::Error as RpcClientError,  XtStatus, ac_primitives::PlainTipExtrinsicParams, rpc::Result as RpcResult
/// };
/// use serde::de::DeserializeOwned;
/// use ac_primitives::RpcParams;
/// use serde_json::{Value, json};
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
	Runtime: FrameSystemConfig,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
{
	signer: Option<Signer>,
	genesis_hash: Runtime::Hash,
	metadata: Metadata,
	runtime_version: RuntimeVersion,
	client: Client,
	additional_extrinsic_params: Option<Params::AdditionalParams>,
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
			additional_extrinsic_params: None,
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

	/// Set the additional params.
	pub fn set_additional_params(&mut self, extrinsic_params: Params::AdditionalParams) {
		self.additional_extrinsic_params = Some(extrinsic_params);
	}

	/// Get the extrinsic params with the set additional params. If no additional params are set,
	/// the default is taken.
	pub fn extrinsic_params(&self, nonce: Runtime::Index) -> Params {
		let additional_extrinsic_params =
			self.additional_extrinsic_params.clone().unwrap_or_default();
		<Params as ExtrinsicParams<Runtime::Index, Runtime::Hash>>::new(
			self.runtime_version.spec_version,
			self.runtime_version.transaction_version,
			nonce,
			self.genesis_hash,
			additional_extrinsic_params,
		)
	}
}

impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	/// Create a new Api client with call to the node to retrieve metadata.
	pub fn new(client: Client) -> Result<Self> {
		let genesis_hash = Self::get_genesis_hash(&client)?;
		info!("Got genesis hash: {:?}", genesis_hash);

		let metadata = Self::get_metadata(&client)?;
		debug!("Metadata: {:?}", metadata);

		let runtime_version = Self::get_runtime_version(&client)?;
		info!("Runtime Version: {:?}", runtime_version);

		Ok(Self::new_offline(genesis_hash, metadata, runtime_version, client))
	}

	/// Updates the runtime and metadata of the api via node query.
	// Ideally, this function is called if a substrate update runtime event is encountered.
	pub fn update_runtime(&mut self) -> Result<()> {
		let metadata = Self::get_metadata(&self.client)?;
		debug!("Metadata: {:?}", metadata);

		let runtime_version = Self::get_runtime_version(&self.client)?;
		info!("Runtime Version: {:?}", runtime_version);

		self.metadata = metadata;
		self.runtime_version = runtime_version;
		Ok(())
	}
}

impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Signer: SignExtrinsic<Runtime::AccountId>,
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	/// Get the public part of the api signer account.
	pub fn signer_account(&self) -> Option<&Runtime::AccountId> {
		let pair = self.signer.as_ref()?;
		Some(pair.public_account_id())
	}

	/// Get nonce of self signer account.
	pub fn get_nonce(&self) -> Result<Runtime::Index> {
		let account = self.signer_account().ok_or(Error::NoSigner)?;
		self.get_account_nonce(account)
	}
}

/// Private node query methods. They should be used internally only, because the user should retrieve the data from the struct cache.
/// If an up-to-date query is necessary, cache should be updated beforehand.
impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	/// Get the genesis hash from node via websocket query.
	fn get_genesis_hash(client: &Client) -> Result<Runtime::Hash> {
		let genesis: Option<Runtime::Hash> =
			client.request("chain_getBlockHash", rpc_params![Some(0)])?;
		genesis.ok_or(Error::FetchGenesisHash)
	}

	/// Get runtime version from node via websocket query.
	fn get_runtime_version(client: &Client) -> Result<RuntimeVersion> {
		let version: RuntimeVersion = client.request("state_getRuntimeVersion", rpc_params![])?;
		Ok(version)
	}

	/// Get metadata from node via websocket query.
	fn get_metadata(client: &Client) -> Result<Metadata> {
		let metadata_bytes: Bytes = client.request("state_getMetadata", rpc_params![])?;

		let metadata = RuntimeMetadataPrefixed::decode(&mut metadata_bytes.0.as_slice())?;
		Metadata::try_from(metadata).map_err(|e| e.into())
	}
}
#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		ac_primitives::{GenericAdditionalParams, PlainTipExtrinsicParams},
		rpc::mocks::RpcClientMock,
	};
	use kitchensink_runtime::Runtime;
	use sp_core::{sr25519::Pair, H256};
	use std::{
		collections::{BTreeMap, HashMap},
		fs,
	};

	fn create_mock_api(
		genesis_hash: H256,
		runtime_version: RuntimeVersion,
		metadata: Metadata,
		data: HashMap<String, String>,
	) -> Api<Pair, RpcClientMock, PlainTipExtrinsicParams<Runtime>, Runtime> {
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

		let expected_params = PlainTipExtrinsicParams::<Runtime>::new(
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
		let metadata: RuntimeMetadataPrefixed =
			Decode::decode(&mut encoded_metadata.0.as_slice()).unwrap();
		let metadata = Metadata::try_from(metadata).unwrap();

		let mut changed_metadata = metadata.clone();
		changed_metadata.errors = BTreeMap::default();

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
		assert_ne!(api.metadata.errors, metadata.errors);
		assert_ne!(api.runtime_version, runtime_version);

		// Update runtime.
		api.update_runtime().unwrap();

		// Ensure metadata and runtime version have been updated.
		assert_eq!(api.metadata.errors, metadata.errors);
		assert_eq!(api.runtime_version, runtime_version);
	}
}
