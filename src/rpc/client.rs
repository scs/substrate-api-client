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

use log::{debug, error, info, warn};
use std::sync::mpsc::Sender as ThreadOut;
use ws::{CloseCode, Handler, Handshake, Message, Result, Sender};

#[derive(Debug, PartialEq)]
pub enum XtStatus {
    Finalized,
    InBlock,
    Broadcast,
    Ready,
    Future,
    Error,
    Unknown,
}

pub type OnMessageFn = fn(msg: Message, out: Sender, result: ThreadOut<String>) -> Result<()>;

pub struct RpcClient {
    pub out: Sender,
    pub request: String,
    pub result: ThreadOut<String>,
    pub on_message_fn: OnMessageFn,
}

impl Handler for RpcClient {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        info!("sending request: {}", self.request);
        self.out.send(self.request.clone()).unwrap();
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        (self.on_message_fn)(msg, self.out.clone(), self.result.clone())
    }
}

pub fn on_get_request_msg(msg: Message, out: Sender, result: ThreadOut<String>) -> Result<()> {
    info!("Got get_request_msg {}", msg);
    let retstr = msg.as_text().unwrap();
    let value: serde_json::Value = serde_json::from_str(retstr).unwrap();

    result.send(value["result"].to_string()).unwrap();
    out.close(CloseCode::Normal).unwrap();
    Ok(())
}

pub fn on_subscription_msg(msg: Message, _out: Sender, result: ThreadOut<String>) -> Result<()> {
    info!("got on_subscription_msg {}", msg);
    let retstr = msg.as_text().unwrap();
    let value: serde_json::Value = serde_json::from_str(retstr).unwrap();
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
                        Some(change_set) => result.send(change_set.to_owned()).unwrap(),
                        None => println!("No events happened"),
                    };
                }
                Some("chain_finalizedHead") => {
                    serde_json::to_string(&value["params"]["result"])
                        .map(|head| result.send(head).unwrap())
                        .unwrap_or_else(|_| error!("Could not parse header"));
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
) -> Result<()> {
    let retstr = msg.as_text().unwrap();
    debug!("got msg {}", retstr);
    match parse_status(retstr) {
        (XtStatus::Finalized, val) => end_process(out, result, val),
        (XtStatus::Error, e) => panic!(e.unwrap()),
        (XtStatus::Future, _) => {
            warn!("extrinsic has 'future' status. aborting");
            end_process(out, result, None);
        }
        _ => (),
    };
    Ok(())
}

pub fn on_extrinsic_msg_until_in_block(
    msg: Message,
    out: Sender,
    result: ThreadOut<String>,
) -> Result<()> {
    let retstr = msg.as_text().unwrap();
    debug!("got msg {}", retstr);
    match parse_status(retstr) {
        (XtStatus::Finalized, val) => end_process(out, result, val),
        (XtStatus::InBlock, val) => end_process(out, result, val),
        (XtStatus::Future, _) => end_process(out, result, None),
        (XtStatus::Error, e) => panic!(e.unwrap()),
        _ => (),
    };
    Ok(())
}

pub fn on_extrinsic_msg_until_broadcast(
    msg: Message,
    out: Sender,
    result: ThreadOut<String>,
) -> Result<()> {
    let retstr = msg.as_text().unwrap();
    debug!("got msg {}", retstr);
    match parse_status(retstr) {
        (XtStatus::Finalized, val) => end_process(out, result, val),
        (XtStatus::Broadcast, _) => end_process(out, result, None),
        (XtStatus::Future, _) => end_process(out, result, None),
        (XtStatus::Error, e) => panic!(e.unwrap()),
        _ => (),
    };
    Ok(())
}

pub fn on_extrinsic_msg_until_ready(
    msg: Message,
    out: Sender,
    result: ThreadOut<String>,
) -> Result<()> {
    let retstr = msg.as_text().unwrap();
    debug!("got msg {}", retstr);
    match parse_status(retstr) {
        (XtStatus::Finalized, val) => end_process(out, result, val),
        (XtStatus::Ready, _) => end_process(out, result, None),
        (XtStatus::Future, _) => end_process(out, result, None),
        (XtStatus::Error, e) => panic!(e.unwrap()),
        _ => (),
    };
    Ok(())
}

fn end_process(out: Sender, result: ThreadOut<String>, value: Option<String>) {
    // return result to calling thread
    debug!("Thread end result :{:?} value:{:?}", result, value);
    let val = value.unwrap_or_else(|| "".to_string());
    result.send(val).unwrap();
    out.close(CloseCode::Normal).unwrap();
}

fn parse_status(msg: &str) -> (XtStatus, Option<String>) {
    let value: serde_json::Value = serde_json::from_str(msg).unwrap();
    match value["error"].as_object() {
        Some(obj) => {
            let error_message = obj.get("message").unwrap().as_str().unwrap().to_owned();
            let code = obj.get("code").unwrap().as_i64().unwrap();
            let details = match obj.get("data") {
                Some(d) => d.as_str().unwrap().to_owned(),
                None => "".to_string(),
            };
            (
                XtStatus::Error,
                Some(format!(
                    "extrinsic error code {}: {}: {}",
                    code, error_message, details
                )),
            )
        }
        None => match value["params"]["result"].as_object() {
            Some(obj) => {
                if let Some(hash) = obj.get("finalized") {
                    info!("finalized: {:?}", hash);
                    (XtStatus::Finalized, Some(hash.to_string()))
                } else if let Some(hash) = obj.get("inBlock") {
                    info!("inBlock: {:?}", hash);
                    (XtStatus::InBlock, Some(hash.to_string()))
                } else if let Some(array) = obj.get("broadcast") {
                    info!("broadcast: {:?}", array);
                    (XtStatus::Broadcast, Some(array.to_string()))
                } else {
                    (XtStatus::Unknown, None)
                }
            }
            None => match value["params"]["result"].as_str() {
                Some("ready") => (XtStatus::Ready, None),
                Some("future") => (XtStatus::Future, None),
                Some(&_) => (XtStatus::Unknown, None),
                None => (XtStatus::Unknown, None),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrinsic_status_parsed_correctly() {
        let msg = "{\"jsonrpc\":\"2.0\",\"result\":7185,\"id\":\"3\"}";
        assert_eq!(parse_status(msg), (XtStatus::Unknown, None));

        let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"author_extrinsicUpdate\",\"params\":{\"result\":\"ready\",\"subscription\":7185}}";
        assert_eq!(parse_status(msg), (XtStatus::Ready, None));

        let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"author_extrinsicUpdate\",\"params\":{\"result\":{\"broadcast\":[\"QmfSF4VYWNqNf5KYHpDEdY8Rt1nPUgSkMweDkYzhSWirGY\",\"Qmchhx9SRFeNvqjUK4ZVQ9jH4zhARFkutf9KhbbAmZWBLx\",\"QmQJAqr98EF1X3YfjVKNwQUG9RryqX4Hv33RqGChbz3Ncg\"]},\"subscription\":232}}";
        assert_eq!(
            parse_status(msg),
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
            parse_status(msg),
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
            parse_status(msg),
            (
                XtStatus::Finalized,
                Some(
                    "\"0x934385b11c483498e2b5bca64c2e8ef76ad6c74d3372a05595d3a50caf758d52\""
                        .to_string()
                )
            )
        );

        let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"author_extrinsicUpdate\",\"params\":{\"result\":\"future\",\"subscription\":2}}";
        assert_eq!(parse_status(msg), (XtStatus::Future, None));

        let msg = "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":-32700,\"message\":\"Parse error\"},\"id\":null}";
        assert_eq!(
            parse_status(msg),
            (
                XtStatus::Error,
                Some("extrinsic error code -32700: Parse error: ".into())
            )
        );

        let msg = "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":1010,\"message\":\"Invalid Transaction\",\"data\":\"Bad Signature\"},\"id\":\"4\"}";
        assert_eq!(
            parse_status(msg),
            (
                XtStatus::Error,
                Some("extrinsic error code 1010: Invalid Transaction: Bad Signature".into())
            )
        );

        let msg = "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":1001,\"message\":\"Extrinsic has invalid format.\"},\"id\":\"0\"}";
        assert_eq!(
            parse_status(msg),
            (
                XtStatus::Error,
                Some("extrinsic error code 1001: Extrinsic has invalid format.: ".into())
            )
        );

        let msg = r#"{"jsonrpc":"2.0","error":{"code":1002,"message":"Verification Error: Execution(Wasmi(Trap(Trap { kind: Unreachable })))","data":"RuntimeApi(\"Execution(Wasmi(Trap(Trap { kind: Unreachable })))\")"},"id":"3"}"#;
        assert_eq!(
            parse_status(msg),
            (
                XtStatus::Error,
                Some(
                    "extrinsic error code 1002: Verification Error: Execution(Wasmi(Trap(Trap { kind: Unreachable }))): RuntimeApi(\"Execution(Wasmi(Trap(Trap { kind: Unreachable })))\")".into()
                )
            )
        );
    }
}
