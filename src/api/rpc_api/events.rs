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
	GetBlock, GetStorage, Phase,
};
use ac_node_api::{EventDetails, Events, StaticEvent};
use ac_primitives::{ExtrinsicParams, FrameSystemConfig, StorageChangeSet};
use alloc::vec::Vec;
use codec::Encode;
use log::*;
use serde::de::DeserializeOwned;
use sp_runtime::traits::{Block as BlockTrait, GetRuntimeBlockType, Hash as HashTrait};
pub trait FetchEvents<Client, Hash>
where
	Client: Request,
	Hash: DeserializeOwned,
{
	/// Fetch all block events from node for the given block hash.
	fn fetch_events_from_block(&self, block_hash: Hash) -> Result<Events<Hash>>;

	/// Fetch all assosciated events for a given extrinsic hash and block hash.
	fn fetch_events_for_extrinsic(
		&self,
		block_hash: Hash,
		extrinsic_hash: Hash,
	) -> Result<Vec<EventDetails>>;
}

impl<Signer, Client, Params, Runtime> FetchEvents<Client, Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig + GetRuntimeBlockType,
	Runtime::RuntimeBlock: BlockTrait + DeserializeOwned,
	Runtime::Hashing: HashTrait<Output = Runtime::Hash>,
{
	fn fetch_events_from_block(&self, block_hash: Runtime::Hash) -> Result<Events<Runtime::Hash>> {
		let key = crate::storage_key("System", "Events");
		let event_bytes = self
			.get_opaque_storage_by_key_hash(key, Some(block_hash))?
			.ok_or(Error::BlockNotFound)?;
		let events =
			Events::<Runtime::Hash>::new(self.metadata().clone(), Default::default(), event_bytes);
		Ok(events)
	}

	fn fetch_events_for_extrinsic(
		&self,
		block_hash: Runtime::Hash,
		extrinsic_hash: Runtime::Hash,
	) -> Result<Vec<EventDetails>> {
		let extrinsic_index =
			self.retrieve_extrinsic_index_from_block(block_hash, extrinsic_hash)?;
		let block_events = self.fetch_events_from_block(block_hash)?;
		self.filter_extrinsic_events(block_events, extrinsic_index)
	}
}

// FIXME: This should rather be implemented directly on the
// Subscription return value, rather than the api. Or directly
// subcribe. Should be looked at in #288
// https://github.com/scs/substrate-api-client/issues/288#issuecomment-1346221653
pub trait SubscribeEvents<Client, Hash>
where
	Client: Subscribe,
	Hash: DeserializeOwned,
{
	fn wait_for_event<Ev: StaticEvent>(
		&self,
		subscription: &mut Client::Subscription<StorageChangeSet<Hash>>,
	) -> Result<Ev>;

	fn wait_for_event_details<Ev: StaticEvent>(
		&self,
		subscription: &mut Client::Subscription<StorageChangeSet<Hash>>,
	) -> Result<EventDetails>;
}

impl<Signer, Client, Params, Runtime> SubscribeEvents<Client, Runtime::Hash>
	for Api<Signer, Client, Params, Runtime>
where
	Client: Subscribe,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig,
{
	fn wait_for_event<Ev: StaticEvent>(
		&self,
		subscription: &mut Client::Subscription<StorageChangeSet<Runtime::Hash>>,
	) -> Result<Ev> {
		let maybe_event_details = self.wait_for_event_details::<Ev>(subscription)?;
		maybe_event_details
			.as_event()?
			.ok_or(Error::Other("Could not find the specific event".into()))
	}

	fn wait_for_event_details<Ev: StaticEvent>(
		&self,
		subscription: &mut Client::Subscription<StorageChangeSet<Runtime::Hash>>,
	) -> Result<EventDetails> {
		while let Some(change_set) = subscription.next() {
			let event_bytes = change_set?.changes[0].1.as_ref().unwrap().0.clone();
			let events = Events::<Runtime::Hash>::new(
				self.metadata().clone(),
				Default::default(),
				event_bytes,
			);
			for maybe_event_details in events.iter() {
				let event_details = maybe_event_details?;

				// Check for failed xt and return as Dispatch Error in case we find one.
				// Careful - this reports the first one encountered. This event may belong to another extrinsic
				// than the one that is being waited for.
				event_details.check_if_failed()?;

				let event_metadata = event_details.event_metadata();
				trace!(
					"Found extrinsic: {:?}, {:?}",
					event_metadata.pallet(),
					event_metadata.event()
				);
				if event_metadata.pallet() == Ev::PALLET && event_metadata.event() == Ev::EVENT {
					return Ok(event_details)
				} else {
					trace!("Not the event we are looking for, skipping.")
				}
			}
		}
		Err(Error::NoStream)
	}
}

impl<Signer, Client, Params, Runtime> Api<Signer, Client, Params, Runtime>
where
	Client: Request,
	Params: ExtrinsicParams<Runtime::Index, Runtime::Hash>,
	Runtime: FrameSystemConfig + GetRuntimeBlockType,
	Runtime::RuntimeBlock: BlockTrait + DeserializeOwned,
	Runtime::Hashing: HashTrait<Output = Runtime::Hash>,
{
	/// Retrieve block details from node and search for the position of the given extrinsic.
	fn retrieve_extrinsic_index_from_block(
		&self,
		block_hash: Runtime::Hash,
		extrinsic_hash: Runtime::Hash,
	) -> Result<u32> {
		let block = self.get_block(Some(block_hash))?.ok_or(Error::BlockNotFound)?;
		let xt_index = block
			.extrinsics()
			.iter()
			.position(|xt| {
				let xt_hash = Runtime::Hashing::hash_of(&xt.encode());
				trace!("Looking for: {:?}, got xt_hash {:?}", extrinsic_hash, xt_hash);
				extrinsic_hash == xt_hash
			})
			.ok_or(Error::ExtrinsicNotFound)?;
		Ok(xt_index as u32)
	}

	/// Filter events and return the ones associated to the given extrinsic index.
	fn filter_extrinsic_events(
		&self,
		events: Events<Runtime::Hash>,
		extrinsic_index: u32,
	) -> Result<Vec<EventDetails>> {
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
	use crate::{rpc::mocks::RpcClientMock, AssetTipExtrinsicParams, StorageData};
	use ac_node_api::{metadata::Metadata, test_utils::*};
	use ac_primitives::{Bytes, FrameSystemConfig, RuntimeVersion, SignedBlock};
	use codec::{Decode, Encode};
	use frame_metadata::RuntimeMetadataPrefixed;
	use kitchensink_runtime::{BalancesCall, Runtime, RuntimeCall, UncheckedExtrinsic};
	use scale_info::TypeInfo;
	use sp_core::{crypto::Ss58Codec, sr25519, sr25519::Pair, H256};
	use sp_runtime::{generic::Block, AccountId32, MultiAddress};
	use std::{collections::HashMap, fs};

	#[derive(Clone, Copy, Debug, PartialEq, Decode, Encode, TypeInfo)]
	enum Event {
		A(u8),
		B(bool),
	}

	fn create_mock_api(
		metadata: Metadata,
		data: HashMap<String, String>,
	) -> Api<Pair, RpcClientMock, AssetTipExtrinsicParams<Runtime>, Runtime> {
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

	#[test]
	fn filter_extrinsic_events_works() {
		let metadata = metadata::<Event>();

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
		let _not_assosciated_event_details1 = event_details.next().unwrap().unwrap();
		let assosciated_event_details2 = event_details.next().unwrap().unwrap();
		let assosciated_event_details3 = event_details.next().unwrap().unwrap();
		let _not_assosciated_event_details4 = event_details.next().unwrap().unwrap();
		assert!(event_details.next().is_none());

		let api = create_mock_api(metadata, Default::default());

		let assosciated_events =
			api.filter_extrinsic_events(block_events, extrinsic_index).unwrap();
		assert_eq!(assosciated_events.len(), 2);
		assert_eq!(assosciated_events[0].index(), assosciated_event_details2.index());
		assert_eq!(assosciated_events[1].index(), assosciated_event_details3.index());
	}

	#[test]
	fn fetch_events_from_block_works() {
		let metadata = metadata::<Event>();

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
		let call2 =
			RuntimeCall::Balances(BalancesCall::transfer { dest: bob.clone(), value: 2000 });
		let call3 = RuntimeCall::Balances(BalancesCall::transfer { dest: bob, value: 1000 });

		let xt1: Bytes = UncheckedExtrinsic::new_unsigned(call1).encode().into();
		let xt2: Bytes = UncheckedExtrinsic::new_unsigned(call2).encode().into();
		let xt3: Bytes = UncheckedExtrinsic::new_unsigned(call3).encode().into();

		let xt_hash1 = <Runtime as FrameSystemConfig>::Hashing::hash_of(&xt1.0);
		let xt_hash2 = <Runtime as FrameSystemConfig>::Hashing::hash_of(&xt2.0);
		let xt_hash3 = <Runtime as FrameSystemConfig>::Hashing::hash_of(&xt3.0);

		let block = Block { header: default_header(), extrinsics: vec![xt1, xt2, xt3] };
		let signed_block = SignedBlock { block, justifications: None };
		let data = HashMap::<String, String>::from([(
			"chain_getBlock".to_owned(),
			serde_json::to_string(&signed_block).unwrap(),
		)]);

		// Create api with block as storage data:
		let api = create_mock_api(metadata, data);
		let block_hash = H256::default();

		let index1 = api.retrieve_extrinsic_index_from_block(block_hash, xt_hash1).unwrap();
		let index2 = api.retrieve_extrinsic_index_from_block(block_hash, xt_hash2).unwrap();
		let index3 = api.retrieve_extrinsic_index_from_block(block_hash, xt_hash3).unwrap();

		assert_eq!(index1, 0);
		assert_eq!(index2, 1);
		assert_eq!(index3, 2);
	}
}
