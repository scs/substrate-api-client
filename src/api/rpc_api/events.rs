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
	api::{Api, Error, Result},
	rpc::{HandleSubscription, Request, Subscribe},
	GetChainInfo, GetStorage,
};
use ac_compose_macros::rpc_params;
use ac_node_api::{metadata::Metadata, EventDetails, EventRecord, Events, Phase};
use ac_primitives::config::Config;
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use alloc::{vec, vec::Vec};
use codec::{Decode, Encode};
use core::marker::PhantomData;
use log::*;
use serde::de::DeserializeOwned;
use sp_runtime::traits::{Block as BlockHash, Hash as HashTrait};
use sp_storage::StorageChangeSet;

pub type EventSubscriptionFor<Client, Hash> =
	EventSubscription<<Client as Subscribe>::Subscription<StorageChangeSet<Hash>>, Hash>;

#[maybe_async::maybe_async(?Send)]
pub trait FetchEvents {
	type Hash: Encode + Decode;

	/// Fetch all block events from node for the given block hash.
	async fn fetch_events_from_block(&self, block_hash: Self::Hash) -> Result<Events<Self::Hash>>;

	/// Fetch all associated events for a given extrinsic hash and block hash.
	async fn fetch_events_for_extrinsic(
		&self,
		block_hash: Self::Hash,
		extrinsic_hash: Self::Hash,
	) -> Result<Vec<EventDetails<Self::Hash>>>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> FetchEvents for Api<T, Client>
where
	T: Config,
	Client: Request,
{
	type Hash = T::Hash;

	async fn fetch_events_from_block(&self, block_hash: Self::Hash) -> Result<Events<Self::Hash>> {
		let key = crate::storage_key("System", "Events");
		let event_bytes = self
			.get_opaque_storage_by_key(key, Some(block_hash))
			.await?
			.ok_or(Error::BlockNotFound)?;
		let events =
			Events::<Self::Hash>::new(self.metadata().clone(), Default::default(), event_bytes);
		Ok(events)
	}

	async fn fetch_events_for_extrinsic(
		&self,
		block_hash: Self::Hash,
		extrinsic_hash: Self::Hash,
	) -> Result<Vec<EventDetails<Self::Hash>>> {
		let extrinsic_index =
			self.retrieve_extrinsic_index_from_block(block_hash, extrinsic_hash).await?;
		let block_events = self.fetch_events_from_block(block_hash).await?;
		self.filter_extrinsic_events(block_events, extrinsic_index)
	}
}

/// Wrapper around a Event `StorageChangeSet` subscription.
/// Simplifies the event retrieval from the subscription.
pub struct EventSubscription<Subscription, Hash> {
	pub subscription: Subscription,
	pub metadata: Metadata,
	_phantom: PhantomData<Hash>,
}

impl<Subscription, Hash> EventSubscription<Subscription, Hash> {
	/// Create a new wrapper around the subscription.
	pub fn new(subscription: Subscription, metadata: Metadata) -> Self {
		Self { subscription, metadata, _phantom: Default::default() }
	}

	/// Update the metadata.
	pub fn update_metadata(&mut self, metadata: Metadata) {
		self.metadata = metadata
	}
}

impl<Subscription, Hash> EventSubscription<Subscription, Hash>
where
	Hash: DeserializeOwned + Copy + Encode + Decode,
	Subscription: HandleSubscription<StorageChangeSet<Hash>>,
{
	/// Wait for the next value from the internal subscription.
	/// Upon encounter, it retrieves and decodes the expected `EventRecord`.
	#[maybe_async::maybe_async(?Send)]
	pub async fn next_events<RuntimeEvent: Decode, Topic: Decode>(
		&mut self,
	) -> Option<Result<Vec<EventRecord<RuntimeEvent, Topic>>>> {
		let change_set = match self.subscription.next().await? {
			Ok(set) => set,
			Err(e) => return Some(Err(Error::RpcClient(e))),
		};
		// Since we subscribed to only the events key, we can simply take the first value of the
		// changes in the set. Also, we don't care about the key but only the data, so take
		// the second value in the tuple of two.
		let storage_data = change_set.changes[0].1.as_ref()?;
		let event_records = Decode::decode(&mut storage_data.0.as_slice()).map_err(Error::Codec);
		Some(event_records)
	}

	/// Wait for the next value from the internal subscription.
	/// Upon encounter, it retrieves and decodes the expected `EventDetails`.
	//
	// On the contrary to `next_events` this function only needs up-to-date metadata
	// and is therefore updateable during runtime.
	#[maybe_async::maybe_async(?Send)]
	pub async fn next_events_from_metadata(&mut self) -> Option<Result<Events<Hash>>> {
		let change_set = match self.subscription.next().await? {
			Ok(set) => set,
			Err(e) => return Some(Err(Error::RpcClient(e))),
		};
		let block_hash = change_set.block;
		// Since we subscribed to only the events key, we can simply take the first value of the
		// changes in the set. Also, we don't care about the key but only the data, so take
		// the second value in the tuple of two.
		let storage_data = change_set.changes[0].1.as_ref()?;
		let event_bytes = storage_data.0.clone();

		let events = Events::<Hash>::new(self.metadata.clone(), block_hash, event_bytes);
		Some(Ok(events))
	}

	/// Unsubscribe from the internal subscription.
	#[maybe_async::maybe_async(?Send)]
	pub async fn unsubscribe(self) -> Result<()> {
		self.subscription.unsubscribe().await.map_err(|e| e.into())
	}
}

#[maybe_async::maybe_async(?Send)]
pub trait SubscribeEvents {
	type Client: Subscribe;
	type Hash: DeserializeOwned;

	/// Subscribe to events.
	async fn subscribe_events(&self) -> Result<EventSubscriptionFor<Self::Client, Self::Hash>>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> SubscribeEvents for Api<T, Client>
where
	T: Config,
	Client: Subscribe,
{
	type Client = Client;
	type Hash = T::Hash;

	async fn subscribe_events(&self) -> Result<EventSubscriptionFor<Self::Client, Self::Hash>> {
		let key = crate::storage_key("System", "Events");
		let subscription = self
			.client()
			.subscribe("state_subscribeStorage", rpc_params![vec![key]], "state_unsubscribeStorage")
			.await
			.map(|sub| EventSubscription::new(sub, self.metadata().clone()))?;
		Ok(subscription)
	}
}

impl<T, Client> Api<T, Client>
where
	T: Config,
	Client: Request,
{
	/// Retrieve block details from node and search for the position of the given extrinsic.
	#[maybe_async::maybe_async(?Send)]
	async fn retrieve_extrinsic_index_from_block(
		&self,
		block_hash: T::Hash,
		extrinsic_hash: T::Hash,
	) -> Result<u32> {
		let block = self.get_block(Some(block_hash)).await?.ok_or(Error::BlockNotFound)?;
		let xt_index = block
			.extrinsics()
			.iter()
			.position(|xt| {
				let xt_hash = T::Hasher::hash_of(&xt);
				trace!("Looking for: {:?}, got xt_hash {:?}", extrinsic_hash, xt_hash);
				extrinsic_hash == xt_hash
			})
			.ok_or(Error::ExtrinsicNotFound)?;
		Ok(xt_index as u32)
	}

	/// Filter events and return the ones associated to the given extrinsic index.
	fn filter_extrinsic_events(
		&self,
		events: Events<T::Hash>,
		extrinsic_index: u32,
	) -> Result<Vec<EventDetails<T::Hash>>> {
		let extrinsic_event_results = events.iter().filter(|ev| {
			ev.as_ref()
				.map_or(true, |ev| ev.phase() == Phase::ApplyExtrinsic(extrinsic_index))
		});
		let mut extrinsic_events = Vec::new();
		for event_details in extrinsic_event_results {
			let event_details = event_details?;
			debug!(
				"associated event_details {:?} {:?}",
				event_details.pallet_name(),
				event_details.variant_name()
			);
			extrinsic_events.push(event_details);
		}
		Ok(extrinsic_events)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::rpc::mocks::RpcClientMock;
	use ac_node_api::{metadata::Metadata, test_utils::*};
	use ac_primitives::DefaultRuntimeConfig;
	use codec::{Decode, Encode};
	use frame_metadata::RuntimeMetadataPrefixed;
	use kitchensink_runtime::{BalancesCall, RuntimeCall, UncheckedExtrinsic};
	use scale_info::TypeInfo;
	use sp_core::{crypto::Ss58Codec, sr25519, Bytes, H256};
	use sp_runtime::{
		generic::{Block, SignedBlock},
		AccountId32, MultiAddress,
	};
	use sp_storage::StorageData;
	use sp_version::RuntimeVersion;
	use std::{collections::HashMap, fs};
	use test_case::test_case;

	#[derive(Clone, Copy, Debug, PartialEq, Decode, Encode, TypeInfo)]
	enum Event {
		A(u8),
		B(bool),
	}

	fn create_mock_api(
		metadata: Metadata,
		data: HashMap<String, String>,
	) -> Api<DefaultRuntimeConfig, RpcClientMock> {
		// Create new api.
		let genesis_hash = H256::random();
		let runtime_version = RuntimeVersion::default();
		let client = RpcClientMock::new(data);
		Api::new_offline(genesis_hash, metadata, runtime_version, client)
	}

	fn default_header() -> kitchensink_runtime::Header {
		kitchensink_runtime::Header {
			number: Default::default(),
			parent_hash: Default::default(),
			state_root: Default::default(),
			extrinsics_root: Default::default(),
			digest: Default::default(),
		}
	}

	#[test_case(SupportedMetadataVersions::V14)]
	#[test_case(SupportedMetadataVersions::V15)]
	fn filter_extrinsic_events_works(metadata_version: SupportedMetadataVersions) {
		let metadata = metadata_with_version::<Event>(metadata_version);

		let extrinsic_index = 1;

		// Random events
		let event1 = Event::A(1);
		let event2 = Event::B(true);
		let event3 = Event::A(234);
		let event4 = Event::A(2);

		let block_events = events::<Event>(
			metadata.clone(),
			vec![
				event_record(Phase::Initialization, event1),
				event_record(Phase::ApplyExtrinsic(extrinsic_index), event2),
				event_record(Phase::ApplyExtrinsic(extrinsic_index), event3),
				event_record(Phase::ApplyExtrinsic(extrinsic_index + 1), event4),
			],
		);
		let mut event_details = block_events.iter();
		let _not_associated_event_details1 = event_details.next().unwrap().unwrap();
		let associated_event_details2 = event_details.next().unwrap().unwrap();
		let associated_event_details3 = event_details.next().unwrap().unwrap();
		let _not_associated_event_details4 = event_details.next().unwrap().unwrap();
		assert!(event_details.next().is_none());

		let api = create_mock_api(metadata, Default::default());

		let associated_events = api.filter_extrinsic_events(block_events, extrinsic_index).unwrap();
		assert_eq!(associated_events.len(), 2);
		assert_eq!(associated_events[0].index(), associated_event_details2.index());
		assert_eq!(associated_events[1].index(), associated_event_details3.index());
	}

	#[test_case(SupportedMetadataVersions::V14)]
	#[test_case(SupportedMetadataVersions::V15)]
	fn fetch_events_from_block_works(metadata_version: SupportedMetadataVersions) {
		let metadata = metadata_with_version::<Event>(metadata_version);

		let extrinsic_index = 1;

		// Random events
		let event1 = Event::A(1);
		let event2 = Event::B(true);
		let event3 = Event::A(234);
		let event4 = Event::A(2);

		let block_events = events::<Event>(
			metadata.clone(),
			vec![
				event_record(Phase::Initialization, event1),
				event_record(Phase::ApplyExtrinsic(extrinsic_index), event2),
				event_record(Phase::ApplyExtrinsic(extrinsic_index), event3),
				event_record(Phase::ApplyExtrinsic(extrinsic_index + 1), event4),
			],
		);
		let event_bytes = block_events.event_bytes().to_vec();

		// With this test, the storage key generation is not tested. This is part
		// of the system test. Therefore, the storage key is simply set to "state_getStorage",
		// without extra params.
		let data = HashMap::<String, String>::from([(
			"state_getStorage".to_owned(),
			serde_json::to_string(&Some(StorageData(event_bytes))).unwrap(),
		)]);

		let api = create_mock_api(metadata, data);

		let fetched_events = api.fetch_events_from_block(H256::random()).unwrap();

		assert_eq!(fetched_events.event_bytes(), block_events.event_bytes());
	}

	#[test]
	fn retrieve_extrinsic_index_from_block_works() {
		// We need a pallet balance in the metadata, so ` api.balance_transfer` can create the extrinsic.
		let encoded_metadata = fs::read("./ksm_metadata_v14.bin").unwrap();
		let metadata: RuntimeMetadataPrefixed =
			Decode::decode(&mut encoded_metadata.as_slice()).unwrap();
		let metadata = Metadata::try_from(metadata).unwrap();

		let bob: AccountId32 =
			sr25519::Public::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")
				.unwrap()
				.into();
		let bob = MultiAddress::Id(bob);

		let call1 = RuntimeCall::Balances(BalancesCall::force_transfer {
			source: bob.clone(),
			dest: bob.clone(),
			value: 10,
		});
		let call2 = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
			dest: bob.clone(),
			value: 2000,
		});
		let call3 =
			RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: bob, value: 1000 });

		let xt1: Bytes = UncheckedExtrinsic::new_unsigned(call1).encode().into();
		let xt2: Bytes = UncheckedExtrinsic::new_unsigned(call2).encode().into();
		let xt3: Bytes = UncheckedExtrinsic::new_unsigned(call3).encode().into();

		let xt_hash1 = <DefaultRuntimeConfig as Config>::Hasher::hash(&xt1);
		let xt_hash2 = <DefaultRuntimeConfig as Config>::Hasher::hash(&xt2);
		let xt_hash3 = <DefaultRuntimeConfig as Config>::Hasher::hash(&xt3);

		let block = Block { header: default_header(), extrinsics: vec![xt1, xt2, xt3] };
		let signed_block = SignedBlock { block, justifications: None };
		let data = HashMap::<String, String>::from([(
			"chain_getBlock".to_owned(),
			serde_json::to_string(&signed_block).unwrap(),
		)]);

		// Create api with block as storage data:
		let api = create_mock_api(metadata, data);
		let block_hash = H256::default();

		let (index1, index2, index3) = (
			api.retrieve_extrinsic_index_from_block(block_hash, xt_hash1).unwrap(),
			api.retrieve_extrinsic_index_from_block(block_hash, xt_hash2).unwrap(),
			api.retrieve_extrinsic_index_from_block(block_hash, xt_hash3).unwrap(),
		);

		assert_eq!(index1, 0);
		assert_eq!(index2, 1);
		assert_eq!(index3, 2);
	}
}
