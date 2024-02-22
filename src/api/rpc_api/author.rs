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

//! Interface to common author rpc functions and helpers thereof.

use crate::{
	api::{rpc_api::events::FetchEvents, Error, Result},
	error::FailedExtrinsicError,
	rpc::{HandleSubscription, Request, Subscribe},
	Api, ExtrinsicReport, TransactionStatus, XtStatus,
};
use ac_compose_macros::rpc_params;
use ac_primitives::{config::Config, UncheckedExtrinsicV4};
#[cfg(not(feature = "sync-api"))]
use alloc::boxed::Box;
use codec::{Decode, Encode};
use log::*;
use serde::de::DeserializeOwned;
use sp_core::Bytes;
use sp_runtime::traits::Hash as HashTrait;

pub type TransactionSubscriptionFor<Client, Hash> =
	<Client as Subscribe>::Subscription<TransactionStatus<Hash, Hash>>;

/// Simple extrinsic submission without any subscription.
#[maybe_async::maybe_async(?Send)]
pub trait SubmitExtrinsic {
	type Hash;

	/// Submit an encodable extrinsic to the substrate node.
	/// Returns the extrinsic hash.
	async fn submit_extrinsic<Address, Call, Signature, SignedExtra>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
	) -> Result<Self::Hash>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode;

	/// Submit an encoded, opaque extrinsic to the substrate node.
	/// Returns the extrinsic hash.
	async fn submit_opaque_extrinsic(&self, encoded_extrinsic: &Bytes) -> Result<Self::Hash>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> SubmitExtrinsic for Api<T, Client>
where
	T: Config,
	Client: Request,
{
	type Hash = T::Hash;

	async fn submit_extrinsic<Address, Call, Signature, SignedExtra>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
	) -> Result<Self::Hash>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode,
	{
		self.submit_opaque_extrinsic(&extrinsic.encode().into()).await
	}

	async fn submit_opaque_extrinsic(&self, encoded_extrinsic: &Bytes) -> Result<Self::Hash> {
		let hex_encoded_xt = rpc_params![encoded_extrinsic];
		debug!("sending extrinsic: {:?}", hex_encoded_xt);
		let xt_hash = self.client().request("author_submitExtrinsic", hex_encoded_xt).await?;
		Ok(xt_hash)
	}
}

#[maybe_async::maybe_async(?Send)]
pub trait SubmitAndWatch {
	type Client: Subscribe;
	type Hash: DeserializeOwned + Decode + Encode;

	/// Submit an extrinsic an return a Subscription
	/// to watch the extrinsic progress.
	///
	/// This method is blocking if the sync-api feature is activated
	async fn submit_and_watch_extrinsic<Address, Call, Signature, SignedExtra>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
	) -> Result<TransactionSubscriptionFor<Self::Client, Self::Hash>>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode;

	/// Submit an encoded, opaque extrinsic an return a Subscription to
	/// watch the extrinsic progress.
	///
	/// This method is blocking if the sync-api feature is activated
	async fn submit_and_watch_opaque_extrinsic(
		&self,
		encoded_extrinsic: &Bytes,
	) -> Result<TransactionSubscriptionFor<Self::Client, Self::Hash>>;

	/// Submit an extrinsic and watch it until the desired status
	/// is reached, if no error is encountered previously.
	///
	/// If watched until `InBlock` or `Finalized`, this function will
	/// return an error if the extrinsic was not successfully executed.
	/// If it was successful, a report containing the following is returned:
	/// - extrinsic hash
	/// - hash of the block the extrinsic was included in
	/// - last known extrinsic (transaction) status
	/// - associated events of the extrinsic
	///
	/// If not watched until at least `InBlock`, this function will not know if the extrinsic
	/// has been executed on chain or not and will therefore not return an error if execution fails.
	/// An error will be returned if the extrinsic has failed to be sent or if it has not been
	/// included into the transaction pool of the node.
	/// If no error occurs, a report containing the following is returned:
	/// - extrinsic hash
	/// - last known extrinsic (transaction) status
	///
	/// This method is blocking if the sync-api feature is activated
	async fn submit_and_watch_extrinsic_until<Address, Call, Signature, SignedExtra>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
		watch_until: XtStatus,
	) -> Result<ExtrinsicReport<Self::Hash>>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode;

	/// Submit an encoded, opaque extrinsic until the desired status
	/// is reached, if no error is encountered previously.
	///
	/// If watched until `InBlock` or `Finalized`, this function will
	/// return an error if the extrinsic was not successfully executed.
	/// If it was successful, a report containing the following is returned:
	/// - extrinsic hash
	/// - hash of the block the extrinsic was included in
	/// - last known extrinsic (transaction) status
	/// - associated events of the extrinsic (only for InBlock or Finalized)
	///
	/// If not watched until at least `InBlock`, this function will not know if the extrinsic
	/// has been executed on chain or not and will therefore not return an error if execution fails..
	/// An error will be returned, if the extrinsic has failed to be sent or if it has not been
	/// included into the transaction pool of the node.
	/// If no error occurs, a report containing the following is returned:
	/// - extrinsic hash
	/// - last known extrinsic (transaction) status
	///
	/// This method is blocking if the sync-api feature is activated
	async fn submit_and_watch_opaque_extrinsic_until(
		&self,
		encoded_extrinsic: &Bytes,
		watch_until: XtStatus,
	) -> Result<ExtrinsicReport<Self::Hash>>;

	/// Submit an extrinsic and watch it until the desired status
	/// is reached, if no error is encountered previously.
	/// The events are not fetched. So no events are listed in the report.
	/// To fetch the triggered events, please use submit_and_watch_extrinsic_until.
	/// Upon success, a report containing the following information is returned:
	/// - extrinsic hash
	/// - if watched until at least `InBlock`:
	///   hash of the block the extrinsic was included in
	/// - last known extrinsic (transaction) status
	///
	/// This method is blocking if the sync-api feature is activated
	async fn submit_and_watch_extrinsic_until_without_events<
		Address,
		Call,
		Signature,
		SignedExtra,
	>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
		watch_until: XtStatus,
	) -> Result<ExtrinsicReport<Self::Hash>>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode;

	/// Submit an encoded, opaque extrinsic and watch it until the desired status
	/// is reached, if no error is encountered previously.
	/// The events are not fetched. So no events are listed in the report.
	/// To fetch the triggered events, please use submit_and_watch_opaque_extrinsic_until.
	/// Upon success, a report containing the following information is returned:
	/// - extrinsic hash
	/// - if watched until at least `InBlock`:
	///   hash of the block the extrinsic was included in
	/// - last known extrinsic (transaction) status
	///
	/// This method is blocking if the sync-api feature is activated
	async fn submit_and_watch_opaque_extrinsic_until_without_events(
		&self,
		encoded_extrinsic: &Bytes,
		watch_until: XtStatus,
	) -> Result<ExtrinsicReport<Self::Hash>>;
}

#[maybe_async::maybe_async(?Send)]
impl<T, Client> SubmitAndWatch for Api<T, Client>
where
	T: Config,
	Client: Subscribe + Request,
{
	type Client = Client;
	type Hash = T::Hash;

	async fn submit_and_watch_extrinsic<Address, Call, Signature, SignedExtra>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
	) -> Result<TransactionSubscriptionFor<Self::Client, Self::Hash>>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode,
	{
		self.submit_and_watch_opaque_extrinsic(&extrinsic.encode().into()).await
	}

	async fn submit_and_watch_opaque_extrinsic(
		&self,
		encoded_extrinsic: &Bytes,
	) -> Result<TransactionSubscriptionFor<Self::Client, Self::Hash>> {
		self.client()
			.subscribe(
				"author_submitAndWatchExtrinsic",
				rpc_params![encoded_extrinsic],
				"author_unsubmitAndWatchExtrinsic",
			)
			.await
			.map_err(|e| e.into())
	}

	async fn submit_and_watch_extrinsic_until<Address, Call, Signature, SignedExtra>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
		watch_until: XtStatus,
	) -> Result<ExtrinsicReport<Self::Hash>>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode,
	{
		self.submit_and_watch_opaque_extrinsic_until(&extrinsic.encode().into(), watch_until)
			.await
	}

	async fn submit_and_watch_opaque_extrinsic_until(
		&self,
		encoded_extrinsic: &Bytes,
		watch_until: XtStatus,
	) -> Result<ExtrinsicReport<Self::Hash>> {
		let mut report = self
			.submit_and_watch_opaque_extrinsic_until_without_events(encoded_extrinsic, watch_until)
			.await?;

		if watch_until < XtStatus::InBlock {
			return Ok(report)
		}
		let block_hash = report.block_hash.ok_or(Error::BlockHashNotFound)?;
		let extrinsic_events =
			self.fetch_events_for_extrinsic(block_hash, report.extrinsic_hash).await?;

		// Check if the extrinsic was succesfull or not.
		let mut maybe_dispatch_error = None;
		for event in &extrinsic_events {
			if let Some(dispatch_error) = event.get_associated_dispatch_error() {
				maybe_dispatch_error = Some(dispatch_error);
				break
			}
		}

		report.events = Some(extrinsic_events.into_iter().map(|event| event.to_raw()).collect());

		if let Some(dispatch_error) = maybe_dispatch_error {
			return Err(Error::FailedExtrinsic(FailedExtrinsicError::new(
				dispatch_error,
				report.encode(),
			)))
		}

		Ok(report)
	}

	async fn submit_and_watch_extrinsic_until_without_events<
		Address,
		Call,
		Signature,
		SignedExtra,
	>(
		&self,
		extrinsic: UncheckedExtrinsicV4<Address, Call, Signature, SignedExtra>,
		watch_until: XtStatus,
	) -> Result<ExtrinsicReport<Self::Hash>>
	where
		Address: Encode,
		Call: Encode,
		Signature: Encode,
		SignedExtra: Encode,
	{
		self.submit_and_watch_opaque_extrinsic_until_without_events(
			&extrinsic.encode().into(),
			watch_until,
		)
		.await
	}

	async fn submit_and_watch_opaque_extrinsic_until_without_events(
		&self,
		encoded_extrinsic: &Bytes,
		watch_until: XtStatus,
	) -> Result<ExtrinsicReport<Self::Hash>> {
		let tx_hash = T::Hasher::hash(encoded_extrinsic);
		let mut subscription: TransactionSubscriptionFor<Self::Client, Self::Hash> =
			self.submit_and_watch_opaque_extrinsic(encoded_extrinsic).await?;

		while let Some(transaction_status) = subscription.next().await {
			let transaction_status = transaction_status?;
			match transaction_status.is_expected() {
				Ok(_) =>
					if transaction_status.reached_status(watch_until) {
						subscription.unsubscribe().await?;
						let block_hash = transaction_status.get_maybe_block_hash();
						return Ok(ExtrinsicReport::new(
							tx_hash,
							block_hash.copied(),
							transaction_status,
							None,
						))
					},
				Err(e) => {
					subscription.unsubscribe().await?;
					return Err(e)
				},
			}
		}
		Err(Error::NoStream)
	}
}
