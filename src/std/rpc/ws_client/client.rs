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

use super::HandleMessage;
use crate::{
	rpc::ws_client::{
		GetRequestHandler, SubmitAndWatchHandler, SubmitOnlyHandler, SubscriptionHandler,
	},
	std::{
		rpc::{
			json_req,
			ws_client::{RpcClient, Subscriber},
		},
		ApiResult, FromHexString, RpcClient as RpcClientTrait, XtStatus,
	},
};
use log::info;
use serde_json::Value;
use sp_core::H256 as Hash;
use std::{
	fmt::Debug,
	sync::mpsc::{channel, Sender as ThreadOut},
	thread,
};
use ws::{connect, Result as WsResult};

#[derive(Debug, Clone)]
pub struct WsRpcClient {
	url: String,
}

impl WsRpcClient {
	pub fn new(url: &str) -> WsRpcClient {
		WsRpcClient { url: url.to_string() }
	}
}

impl RpcClientTrait for WsRpcClient {
	fn get_request(&self, jsonreq: Value) -> ApiResult<String> {
		Ok(self
			.direct_rpc_request(jsonreq.to_string(), GetRequestHandler::default())??
			.unwrap_or_default())
	}

	fn send_extrinsic(
		&self,
		xthex_prefixed: String,
		exit_on: XtStatus,
	) -> ApiResult<Option<sp_core::H256>> {
		// Todo: Make all variants return a H256: #175.

		let jsonreq = match exit_on {
			XtStatus::SubmitOnly => json_req::author_submit_extrinsic(&xthex_prefixed).to_string(),
			_ => json_req::author_submit_and_watch_extrinsic(&xthex_prefixed).to_string(),
		};

		let maybe_response =
			self.direct_rpc_request(jsonreq, SubmitAndWatchHandler::new(exit_on))??;
		info!("Got response {:?} while waiting for {:?}", maybe_response, exit_on);
		match maybe_response {
			Some(response) => Ok(Some(Hash::from_hex(response)?)),
			None => Ok(None),
		}
	}
}

impl Subscriber for WsRpcClient {
	fn start_subscriber(
		&self,
		json_req: String,
		result_in: ThreadOut<<SubscriptionHandler as HandleMessage>::ThreadMessage>,
	) -> Result<(), ws::Error> {
		self.start_subscriber(json_req, result_in)
	}
}

#[allow(clippy::result_large_err)]
impl WsRpcClient {
	pub fn get(
		&self,
		json_req: String,
		result_in: ThreadOut<<GetRequestHandler as HandleMessage>::ThreadMessage>,
	) -> WsResult<()> {
		self.start_rpc_client_thread(json_req, result_in, GetRequestHandler::default())
	}

	pub fn send_extrinsic(
		&self,
		json_req: String,
		result_in: ThreadOut<<SubmitOnlyHandler as HandleMessage>::ThreadMessage>,
	) -> WsResult<()> {
		self.start_rpc_client_thread(json_req, result_in, SubmitOnlyHandler::default())
	}

	pub fn send_extrinsic_until_ready(
		&self,
		json_req: String,
		result_in: ThreadOut<<SubmitAndWatchHandler as HandleMessage>::ThreadMessage>,
	) -> WsResult<()> {
		self.start_rpc_client_thread(
			json_req,
			result_in,
			SubmitAndWatchHandler::new(XtStatus::Ready),
		)
	}

	pub fn send_extrinsic_and_wait_until_broadcast(
		&self,
		json_req: String,
		result_in: ThreadOut<<SubmitAndWatchHandler as HandleMessage>::ThreadMessage>,
	) -> WsResult<()> {
		self.start_rpc_client_thread(
			json_req,
			result_in,
			SubmitAndWatchHandler::new(XtStatus::Broadcast),
		)
	}

	pub fn send_extrinsic_and_wait_until_in_block(
		&self,
		json_req: String,
		result_in: ThreadOut<<SubmitAndWatchHandler as HandleMessage>::ThreadMessage>,
	) -> WsResult<()> {
		self.start_rpc_client_thread(
			json_req,
			result_in,
			SubmitAndWatchHandler::new(XtStatus::InBlock),
		)
	}

	pub fn send_extrinsic_and_wait_until_finalized(
		&self,
		json_req: String,
		result_in: ThreadOut<<SubmitAndWatchHandler as HandleMessage>::ThreadMessage>,
	) -> WsResult<()> {
		self.start_rpc_client_thread(
			json_req,
			result_in,
			SubmitAndWatchHandler::new(XtStatus::Finalized),
		)
	}

	pub fn start_subscriber(
		&self,
		json_req: String,
		result_in: ThreadOut<<SubscriptionHandler as HandleMessage>::ThreadMessage>,
	) -> WsResult<()> {
		self.start_rpc_client_thread(json_req, result_in, SubscriptionHandler::default())
	}

	fn start_rpc_client_thread<MessageHandler>(
		&self,
		jsonreq: String,
		result_in: ThreadOut<MessageHandler::ThreadMessage>,
		message_handler: MessageHandler,
	) -> WsResult<()>
	where
		MessageHandler: HandleMessage + Clone + Send + 'static,
		MessageHandler::ThreadMessage: Send + Sync + Debug,
	{
		let url = self.url.clone();
		let _client =
			thread::Builder::new()
				.name("client".to_owned())
				.spawn(move || -> WsResult<()> {
					connect(url, |out| RpcClient {
						out,
						request: jsonreq.clone(),
						result: result_in.clone(),
						message_handler: message_handler.clone(),
					})
				})?;
		Ok(())
	}

	fn direct_rpc_request<MessageHandler>(
		&self,
		jsonreq: String,
		message_handler: MessageHandler,
	) -> ApiResult<MessageHandler::ThreadMessage>
	where
		MessageHandler: HandleMessage + Clone + Send + 'static,
		MessageHandler::ThreadMessage: Send + Sync + Debug,
	{
		let (result_in, result_out) = channel();
		connect(self.url.as_str(), |out| RpcClient {
			out,
			request: jsonreq.clone(),
			result: result_in.clone(),
			message_handler: message_handler.clone(),
		})?;
		Ok(result_out.recv()?)
	}
}
