use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as ThreadOut;
use std::thread;

use log::info;
use serde_json::Value;
use sp_core::H256 as Hash;
use ws::{connect, Result as WsResult};

use crate::rpc::ws_client::on_extrinsic_msg_submit_only;
use crate::std::rpc::json_req;
use crate::std::rpc::ws_client::Subscriber;
use crate::std::rpc::ws_client::{
    on_extrinsic_msg_until_broadcast, on_extrinsic_msg_until_finalized,
    on_extrinsic_msg_until_in_block, on_extrinsic_msg_until_ready, on_get_request_msg,
    on_subscription_msg, OnMessageFn, RpcClient,
};
use crate::std::ApiClientError;
use crate::std::ApiResult;
use crate::std::FromHexString;
use crate::std::RpcClient as RpcClientTrait;
use crate::std::XtStatus;

#[derive(Debug, Clone)]
pub struct WsRpcClient {
    url: String,
}

impl WsRpcClient {
    pub fn new(url: &str) -> WsRpcClient {
        WsRpcClient {
            url: url.to_string(),
        }
    }
}

impl RpcClientTrait for WsRpcClient {
    fn get_request(&self, jsonreq: Value) -> ApiResult<String> {
        self.direct_rpc_request(jsonreq.to_string(), on_get_request_msg)
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

        match exit_on {
            XtStatus::Finalized => {
                let res = self.direct_rpc_request(jsonreq, on_extrinsic_msg_until_finalized)?;
                info!("finalized: {}", res);
                Ok(Some(Hash::from_hex(res)?))
            }
            XtStatus::InBlock => {
                let res = self.direct_rpc_request(jsonreq, on_extrinsic_msg_until_in_block)?;
                info!("inBlock: {}", res);
                Ok(Some(Hash::from_hex(res)?))
            }
            XtStatus::Broadcast => {
                let res = self.direct_rpc_request(jsonreq, on_extrinsic_msg_until_broadcast)?;
                info!("broadcast: {}", res);
                Ok(None)
            }
            XtStatus::Ready => {
                let res = self.direct_rpc_request(jsonreq, on_extrinsic_msg_until_ready)?;
                info!("ready: {}", res);
                Ok(None)
            }
            XtStatus::SubmitOnly => {
                let res = self.direct_rpc_request(jsonreq, on_extrinsic_msg_submit_only)?;
                info!("submitted xt: {}", res);
                Ok(None)
            }
            _ => Err(ApiClientError::UnsupportedXtStatus(exit_on)),
        }
    }
}

impl Subscriber for WsRpcClient {
    fn start_subscriber(
        &self,
        json_req: String,
        result_in: ThreadOut<String>,
    ) -> Result<(), ws::Error> {
        self.start_subscriber(json_req, result_in)
    }
}

impl WsRpcClient {
    pub fn get(&self, json_req: String, result_in: ThreadOut<String>) -> WsResult<()> {
        self.start_rpc_client_thread(json_req, result_in, on_get_request_msg)
    }

    pub fn send_extrinsic(&self, json_req: String, result_in: ThreadOut<String>) -> WsResult<()> {
        self.start_rpc_client_thread(json_req, result_in, on_extrinsic_msg_submit_only)
    }

    pub fn send_extrinsic_until_ready(
        &self,
        json_req: String,
        result_in: ThreadOut<String>,
    ) -> WsResult<()> {
        self.start_rpc_client_thread(json_req, result_in, on_extrinsic_msg_until_ready)
    }

    pub fn send_extrinsic_and_wait_until_broadcast(
        &self,
        json_req: String,
        result_in: ThreadOut<String>,
    ) -> WsResult<()> {
        self.start_rpc_client_thread(json_req, result_in, on_extrinsic_msg_until_broadcast)
    }

    pub fn send_extrinsic_and_wait_until_in_block(
        &self,
        json_req: String,
        result_in: ThreadOut<String>,
    ) -> WsResult<()> {
        self.start_rpc_client_thread(json_req, result_in, on_extrinsic_msg_until_in_block)
    }

    pub fn send_extrinsic_and_wait_until_finalized(
        &self,
        json_req: String,
        result_in: ThreadOut<String>,
    ) -> WsResult<()> {
        self.start_rpc_client_thread(json_req, result_in, on_extrinsic_msg_until_finalized)
    }

    pub fn start_subscriber(&self, json_req: String, result_in: ThreadOut<String>) -> WsResult<()> {
        self.start_rpc_client_thread(json_req, result_in, on_subscription_msg)
    }

    fn start_rpc_client_thread(
        &self,
        jsonreq: String,
        result_in: ThreadOut<String>,
        on_message_fn: OnMessageFn,
    ) -> WsResult<()> {
        let url = self.url.clone();
        let _client =
            thread::Builder::new()
                .name("client".to_owned())
                .spawn(move || -> WsResult<()> {
                    connect(url, |out| RpcClient {
                        out,
                        request: jsonreq.clone(),
                        result: result_in.clone(),
                        on_message_fn,
                    })
                })?;
        Ok(())
    }

    fn direct_rpc_request(&self, jsonreq: String, on_message_fn: OnMessageFn) -> ApiResult<String> {
        let (result_in, result_out) = channel();
        connect(self.url.as_str(), |out| RpcClient {
            out,
            request: jsonreq.clone(),
            result: result_in.clone(),
            on_message_fn,
        })?;
        Ok(result_out.recv()?)
    }
}
