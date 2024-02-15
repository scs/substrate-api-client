/*
   Copyright 2024 Supercomputing Systems AG
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

use super::{RuntimeApi, RuntimeApiClient};
use crate::{api::Result, rpc::Request};
use ac_node_api::{error::MetadataError, Metadata};
use ac_primitives::config::Config;
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use alloc::{
	string::{String, ToString},
	vec,
	vec::Vec,
};
use codec::Decode;
use sp_core::{Encode, OpaqueMetadata};

#[maybe_async::maybe_async(?Send)]
pub trait MetadataApi: RuntimeApi {
	type OpaqueMetadata;

	/// Returns the metadata of a runtime.
	async fn metadata(&self, at_block: Option<Self::Hash>) -> Result<Metadata>;

	/// Returns the opaque metadata of a runtime.
	async fn opaque_metadata(&self, at_block: Option<Self::Hash>) -> Result<Self::OpaqueMetadata>;

	/// Returns the metadata at a given version.
	async fn metadata_at_version(
		&self,
		version: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Metadata>>;

	/// Returns the opaque metadata at a given version.
	async fn opaque_metadata_at_version(
		&self,
		version: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Self::OpaqueMetadata>>;

	/// Returns the supported metadata versions.
	async fn metadata_versions(&self, at_block: Option<Self::Hash>) -> Result<Vec<u32>>;

	// Returns a list of the all available api traits.
	async fn list_traits(&self, at_block: Option<Self::Hash>) -> Result<Vec<String>>;

	// Returns a list of the method names of a specific trait.
	async fn list_methods_of_trait(
		&self,
		trait_name: &str,
		at_block: Option<Self::Hash>,
	) -> Result<Vec<String>>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> MetadataApi for RuntimeApiClient<T, Client>
where
	T: Config,
	Client: Request,
{
	type OpaqueMetadata = OpaqueMetadata;

	async fn metadata(&self, at_block: Option<Self::Hash>) -> Result<Metadata> {
		let metadata_bytes = self.opaque_metadata(at_block).await?;
		let metadata = Metadata::decode(&mut metadata_bytes.as_slice())?;
		Ok(metadata)
	}

	async fn opaque_metadata(&self, at_block: Option<Self::Hash>) -> Result<Self::OpaqueMetadata> {
		self.runtime_call("Metadata_metadata", vec![], at_block).await
	}

	async fn metadata_at_version(
		&self,
		version: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Metadata>> {
		let metadata_bytes = self.opaque_metadata_at_version(version, at_block).await?;
		let metadata = match metadata_bytes {
			Some(bytes) => Some(Metadata::decode(&mut bytes.as_slice())?),
			None => None,
		};
		Ok(metadata)
	}

	async fn opaque_metadata_at_version(
		&self,
		version: u32,
		at_block: Option<Self::Hash>,
	) -> Result<Option<Self::OpaqueMetadata>> {
		self.runtime_call("Metadata_metadata_at_version", vec![version.encode()], at_block)
			.await
	}

	async fn metadata_versions(&self, at_block: Option<Self::Hash>) -> Result<Vec<u32>> {
		self.runtime_call("Metadata_metadata_versions", vec![], at_block).await
	}

	async fn list_traits(&self, at_block: Option<Self::Hash>) -> Result<Vec<String>> {
		let metadata = self.get_metadata_v15(at_block).await?;
		let trait_names = metadata
			.runtime_api_traits()
			.map(|substrate_trait| substrate_trait.name().to_string())
			.collect();

		Ok(trait_names)
	}

	async fn list_methods_of_trait(
		&self,
		trait_name: &str,
		at_block: Option<Self::Hash>,
	) -> Result<Vec<String>> {
		let metadata = self.get_metadata_v15(at_block).await?;
		let maybe_runtime_api_metadata = metadata
			.runtime_api_traits()
			.find(|substrate_trait| substrate_trait.name() == trait_name);

		let methods = match maybe_runtime_api_metadata {
			Some(trait_metadata) =>
				trait_metadata.methods().map(|method| method.name.clone()).collect(),
			None => return Err(MetadataError::RuntimeApiNotFound(trait_name.to_string()).into()),
		};
		Ok(methods)
	}
}

impl<T, Client> RuntimeApiClient<T, Client>
where
	T: Config,
	Client: Request,
{
	#[maybe_async::maybe_async(?Send)]
	async fn get_metadata_v15(&self, at_block: Option<T::Hash>) -> Result<Metadata> {
		self.metadata_at_version(15, at_block)
			.await?
			.ok_or(MetadataError::RuntimeApiNotFound("No metadata v15 found".to_string()).into())
	}
}
