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
use std::sync::mpsc::{Receiver, SendError, Sender as ThreadOut};

use ac_node_api::events::{EventsDecoder, Raw, RawEvent};
use codec::Decode;
use log::{debug, error, info, warn};
use serde_json::Value;
use sp_core::Pair;
use sp_runtime::MultiSignature;
use ws::{CloseCode, Error, Handler, Handshake, Message, Result as WsResult, Sender};

use crate::std::rpc::RpcClientError;
use crate::std::{json_req, FromHexString, RpcClient as RpcClientTrait, XtStatus};
use crate::std::{Api, ApiResult};
use crate::utils;

pub use client::WsRpcClient;

pub mod client;

pub type OnMessageFn = fn(msg: Message, out: Sender, result: ThreadOut<String>) -> WsResult<()>;

type RpcResult<T> = Result<T, RpcClientError>;

pub struct RpcClient {
    pub out: Sender,
    pub request: String,
    pub result: ThreadOut<String>,
    pub on_message_fn: OnMessageFn,
}

impl Handler for RpcClient {
    fn on_open(&mut self, _: Handshake) -> WsResult<()> {
        info!("sending request: {}", self.request);
        self.out.send(self.request.clone())?;
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> WsResult<()> {
        (self.on_message_fn)(msg, self.out.clone(), self.result.clone())
    }
}

pub trait Subscriber {
    fn start_subscriber(&self, json_req: String, result_in: ThreadOut<String>)
        -> Result<(), Error>;
}

impl<P> Api<P, WsRpcClient> {
    pub fn default_with_url(url: &str) -> ApiResult<Self> {
        let client = WsRpcClient::new(url);
        Self::new(client)
    }
}

impl<P, Client> Api<P, Client>
where
    P: Pair,
    MultiSignature: From<P::Signature>,
    Client: RpcClientTrait + Subscriber,
{
    pub fn subscribe_events(&self, sender: ThreadOut<String>) -> ApiResult<()> {
        debug!("subscribing to events");
        let key = utils::storage_key("System", "Events");
        let jsonreq = json_req::state_subscribe_storage(vec![key]).to_string();
        self.client
            .start_subscriber(jsonreq, sender)
            .map_err(|e| e.into())
    }

    pub fn subscribe_finalized_heads(&self, sender: ThreadOut<String>) -> ApiResult<()> {
        debug!("subscribing to finalized heads");
        let jsonreq = json_req::chain_subscribe_finalized_heads().to_string();
        self.client
            .start_subscriber(jsonreq, sender)
            .map_err(|e| e.into())
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
            None => EventsDecoder::new(self.metadata.clone()),
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
                            Raw::Event(raw) if raw.pallet == module && raw.variant == variant => {
                                return Ok(raw);
                            }
                            Raw::Error(runtime_error) => {
                                error!("Some extrinsic Failed: {:?}", runtime_error);
                            }
                            _ => debug!("ignoring unsupported module event: {:?}", event),
                        }
                    }
                }
                Err(error) => error!("couldn't decode event record list: {:?}", error),
            }
        }
    }
}

pub fn on_get_request_msg(msg: Message, out: Sender, result: ThreadOut<String>) -> WsResult<()> {
    out.close(CloseCode::Normal)
        .unwrap_or_else(|_| warn!("Could not close Websocket normally"));

    info!("Got get_request_msg {}", msg);
    let result_str = serde_json::from_str(msg.as_text()?)
        .map(|v: serde_json::Value| v["result"].to_string())
        .map_err(|e| Box::new(RpcClientError::Serde(e)))?;

    result
        .send(result_str)
        .map_err(|e| Box::new(RpcClientError::Send(e)).into())
}

pub fn on_subscription_msg(msg: Message, out: Sender, result: ThreadOut<String>) -> WsResult<()> {
    info!("got on_subscription_msg {}", msg);
    let value: serde_json::Value =
        serde_json::from_str(msg.as_text()?).map_err(|e| Box::new(RpcClientError::Serde(e)))?;

    match value["id"].as_str() {
        Some(_idstr) => {}
        _ => {
            // subscriptions
            debug!("no id field found in response. must be subscription");
            debug!("method: {:?}", value["method"].as_str());
            match value["method"].as_str() {
                Some("state_storage") => {
                    let changes = &value["params"]["result"]["changes"];
                    match changes[0][1].as_str() {
                        Some(change_set) => {
                            if let Err(SendError(e)) = result.send(change_set.to_owned()) {
                                debug!("SendError: {}. will close ws", e);
                                out.close(CloseCode::Normal)?;
                            }
                        }
                        None => println!("No events happened"),
                    };
                }
                Some("chain_finalizedHead") => {
                    let head = serde_json::to_string(&value["params"]["result"])
                        .map_err(|e| Box::new(RpcClientError::Serde(e)))?;

                    if let Err(e) = result.send(head) {
                        debug!("SendError: {}. will close ws", e);
                        out.close(CloseCode::Normal)?;
                    }
                }
                _ => error!("unsupported method"),
            }
        }
    };
    Ok(())
}

pub fn on_extrinsic_msg_until_finalized(
    msg: Message,
    out: Sender,
    result: ThreadOut<String>,
) -> WsResult<()> {
    let retstr = msg.as_text().unwrap();
    debug!("got msg {}", retstr);
    match parse_status(retstr) {
        Ok((XtStatus::Finalized, val)) => end_process(out, result, val),
        Ok((XtStatus::Future, _)) => {
            warn!("extrinsic has 'future' status. aborting");
            end_process(out, result, None)
        }
        Err(e) => {
            end_process(out, result, None)?;
            Err(Box::new(e).into())
        }
        _ => Ok(()),
    }
}

pub fn on_extrinsic_msg_until_in_block(
    msg: Message,
    out: Sender,
    result: ThreadOut<String>,
) -> WsResult<()> {
    let retstr = msg.as_text().unwrap();
    debug!("got msg {}", retstr);
    match parse_status(retstr) {
        Ok((XtStatus::Finalized, val)) => end_process(out, result, val),
        Ok((XtStatus::InBlock, val)) => end_process(out, result, val),
        Ok((XtStatus::Future, _)) => end_process(out, result, None),
        Err(e) => {
            end_process(out, result, None)?;
            Err(Box::new(e).into())
        }
        _ => Ok(()),
    }
}

pub fn on_extrinsic_msg_until_broadcast(
    msg: Message,
    out: Sender,
    result: ThreadOut<String>,
) -> WsResult<()> {
    let retstr = msg.as_text().unwrap();
    debug!("got msg {}", retstr);
    match parse_status(retstr) {
        Ok((XtStatus::Finalized, val)) => end_process(out, result, val),
        Ok((XtStatus::Broadcast, _)) => end_process(out, result, None),
        Ok((XtStatus::Future, _)) => end_process(out, result, None),
        Err(e) => {
            end_process(out, result, None)?;
            Err(Box::new(e).into())
        }
        _ => Ok(()),
    }
}

pub fn on_extrinsic_msg_until_ready(
    msg: Message,
    out: Sender,
    result: ThreadOut<String>,
) -> WsResult<()> {
    let retstr = msg.as_text().unwrap();
    debug!("got msg {}", retstr);
    match parse_status(retstr) {
        Ok((XtStatus::Finalized, val)) => end_process(out, result, val),
        Ok((XtStatus::Ready, _)) => end_process(out, result, None),
        Ok((XtStatus::Future, _)) => end_process(out, result, None),
        Err(e) => {
            end_process(out, result, None)?;
            Err(Box::new(e).into())
        }
        _ => Ok(()),
    }
}

pub fn on_extrinsic_msg_submit_only(
    msg: Message,
    out: Sender,
    result: ThreadOut<String>,
) -> WsResult<()> {
    let retstr = msg.as_text().unwrap();
    debug!("got msg {}", retstr);
    match result_from_json_response(retstr) {
        Ok(val) => end_process(out, result, Some(val)),
        Err(e) => {
            end_process(out, result, None)?;
            Err(Box::new(e).into())
        }
    }
}

fn end_process(out: Sender, result: ThreadOut<String>, value: Option<String>) -> WsResult<()> {
    // return result to calling thread
    debug!("Thread end result :{:?} value:{:?}", result, value);
    let val = value.unwrap_or_else(|| "".to_string());

    out.close(CloseCode::Normal)
        .unwrap_or_else(|_| warn!("Could not close WebSocket normally"));

    result
        .send(val)
        .map_err(|e| Box::new(RpcClientError::Send(e)).into())
}

fn parse_status(msg: &str) -> RpcResult<(XtStatus, Option<String>)> {
    let value: serde_json::Value = serde_json::from_str(msg)?;

    if value["error"].as_object().is_some() {
        return Err(into_extrinsic_err(&value));
    }

    match value["params"]["result"].as_object() {
        Some(obj) => {
            if let Some(hash) = obj.get("finalized") {
                info!("finalized: {:?}", hash);
                Ok((XtStatus::Finalized, Some(hash.to_string())))
            } else if let Some(hash) = obj.get("inBlock") {
                info!("inBlock: {:?}", hash);
                Ok((XtStatus::InBlock, Some(hash.to_string())))
            } else if let Some(array) = obj.get("broadcast") {
                info!("broadcast: {:?}", array);
                Ok((XtStatus::Broadcast, Some(array.to_string())))
            } else {
                Ok((XtStatus::Unknown, None))
            }
        }
        None => match value["params"]["result"].as_str() {
            Some("ready") => Ok((XtStatus::Ready, None)),
            Some("future") => Ok((XtStatus::Future, None)),
            Some(&_) => Ok((XtStatus::Unknown, None)),
            None => Ok((XtStatus::Unknown, None)),
        },
    }
}

/// Todo: this is the code that was used in `parse_status` Don't we want to just print the
/// error as is instead of introducing our custom format here?
fn into_extrinsic_err(resp_with_err: &Value) -> RpcClientError {
    let err_obj = resp_with_err["error"].as_object().unwrap();

    let error = err_obj
        .get("message")
        .map_or_else(|| "", |e| e.as_str().unwrap());
    let code = err_obj
        .get("code")
        .map_or_else(|| -1, |c| c.as_i64().unwrap());
    let details = err_obj
        .get("data")
        .map_or_else(|| "", |d| d.as_str().unwrap());

    RpcClientError::Extrinsic(format!(
        "extrinsic error code {}: {}: {}",
        code, error, details
    ))
}

fn result_from_json_response(resp: &str) -> RpcResult<String> {
    let value: serde_json::Value = serde_json::from_str(resp)?;

    let resp = value["result"]
        .as_str()
        .ok_or_else(|| into_extrinsic_err(&value))?;

    Ok(resp.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::RpcClientError;
    use std::assert_matches::assert_matches;
    use std::fmt::Debug;

    fn assert_extrinsic_err<T: Debug>(result: Result<T, RpcClientError>, msg: &str) {
        assert_matches!(result.unwrap_err(), RpcClientError::Extrinsic(
			m,
		) if &m == msg)
    }

    #[test]
    fn result_from_json_response_works() {
        let msg = r#"{"jsonrpc":"2.0","result":"0xe7640c3e8ba8d10ed7fed07118edb0bfe2d765d3ea2f3a5f6cf781ae3237788f","id":"3"}"#;

        assert_eq!(
            result_from_json_response(msg).unwrap(),
            "0xe7640c3e8ba8d10ed7fed07118edb0bfe2d765d3ea2f3a5f6cf781ae3237788f"
        );
    }

    #[test]
    fn result_from_json_response_errs_on_error_response() {
        let _err_raw =
            r#"{"code":-32602,"message":"Invalid params: invalid hex character: h, at 284."}"#;

        let err_msg = format!(
            "extrinsic error code {}: {}: {}",
            -32602, "Invalid params: invalid hex character: h, at 284.", ""
        );

        let msg = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params: invalid hex character: h, at 284."},"id":"3"}"#;

        assert_extrinsic_err(result_from_json_response(msg), &err_msg)
    }

    #[test]
    fn extrinsic_status_parsed_correctly() {
        let msg = "{\"jsonrpc\":\"2.0\",\"result\":7185,\"id\":\"3\"}";
        assert_eq!(parse_status(msg).unwrap(), (XtStatus::Unknown, None));

        let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"author_extrinsicUpdate\",\"params\":{\"result\":\"ready\",\"subscription\":7185}}";
        assert_eq!(parse_status(msg).unwrap(), (XtStatus::Ready, None));

        let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"author_extrinsicUpdate\",\"params\":{\"result\":{\"broadcast\":[\"QmfSF4VYWNqNf5KYHpDEdY8Rt1nPUgSkMweDkYzhSWirGY\",\"Qmchhx9SRFeNvqjUK4ZVQ9jH4zhARFkutf9KhbbAmZWBLx\",\"QmQJAqr98EF1X3YfjVKNwQUG9RryqX4Hv33RqGChbz3Ncg\"]},\"subscription\":232}}";
        assert_eq!(
            parse_status(msg).unwrap(),
            (
                XtStatus::Broadcast,
                Some(
                    "[\"QmfSF4VYWNqNf5KYHpDEdY8Rt1nPUgSkMweDkYzhSWirGY\",\"Qmchhx9SRFeNvqjUK4ZVQ9jH4zhARFkutf9KhbbAmZWBLx\",\"QmQJAqr98EF1X3YfjVKNwQUG9RryqX4Hv33RqGChbz3Ncg\"]"
                        .to_string()
                )
            )
        );

        let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"author_extrinsicUpdate\",\"params\":{\"result\":{\"inBlock\":\"0x3104d362365ff5ddb61845e1de441b56c6722e94c1aee362f8aa8ba75bd7a3aa\"},\"subscription\":232}}";
        assert_eq!(
            parse_status(msg).unwrap(),
            (
                XtStatus::InBlock,
                Some(
                    "\"0x3104d362365ff5ddb61845e1de441b56c6722e94c1aee362f8aa8ba75bd7a3aa\""
                        .to_string()
                )
            )
        );

        let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"author_extrinsicUpdate\",\"params\":{\"result\":{\"finalized\":\"0x934385b11c483498e2b5bca64c2e8ef76ad6c74d3372a05595d3a50caf758d52\"},\"subscription\":7185}}";
        assert_eq!(
            parse_status(msg).unwrap(),
            (
                XtStatus::Finalized,
                Some(
                    "\"0x934385b11c483498e2b5bca64c2e8ef76ad6c74d3372a05595d3a50caf758d52\""
                        .to_string()
                )
            )
        );

        let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"author_extrinsicUpdate\",\"params\":{\"result\":\"future\",\"subscription\":2}}";
        assert_eq!(parse_status(msg).unwrap(), (XtStatus::Future, None));

        let msg = "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":-32700,\"message\":\"Parse error\"},\"id\":null}";
        assert_extrinsic_err(
            parse_status(msg),
            "extrinsic error code -32700: Parse error: ",
        );

        let msg = "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":1010,\"message\":\"Invalid Transaction\",\"data\":\"Bad Signature\"},\"id\":\"4\"}";
        assert_extrinsic_err(
            parse_status(msg),
            "extrinsic error code 1010: Invalid Transaction: Bad Signature",
        );

        let msg = "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":1001,\"message\":\"Extrinsic has invalid format.\"},\"id\":\"0\"}";
        assert_extrinsic_err(
            parse_status(msg),
            "extrinsic error code 1001: Extrinsic has invalid format.: ",
        );

        let msg = r#"{"jsonrpc":"2.0","error":{"code":1002,"message":"Verification Error: Execution(Wasmi(Trap(Trap { kind: Unreachable })))","data":"RuntimeApi(\"Execution(Wasmi(Trap(Trap { kind: Unreachable })))\")"},"id":"3"}"#;
        assert_extrinsic_err(
            parse_status(msg),
            "extrinsic error code 1002: Verification Error: Execution(Wasmi(Trap(Trap { kind: Unreachable }))): RuntimeApi(\"Execution(Wasmi(Trap(Trap { kind: Unreachable })))\")"
        );
    }
}
