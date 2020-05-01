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
        (XtStatus::Error, e) => end_process(out, result, e),
        (XtStatus::Future, _) => {
            warn!("extrinsic has 'future' status. aborting");
            end_process(out, result, None);
        }
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
        (XtStatus::Error, e) => end_process(out, result, e),
        _ => (),
    };
    Ok(())
}

fn end_process(out: Sender, result: ThreadOut<String>, value: Option<String>) {
    // return result to calling thread
    println!(
        "Thread end result :{:?} value:{:?}",
        result.clone(),
        value.clone()
    );
    let val = value.unwrap_or_else(|| "nix".to_string());
    result.send(val).unwrap();
    out.close(CloseCode::Normal).unwrap();
}

fn parse_status(msg: &str) -> (XtStatus, Option<String>) {
    let value: serde_json::Value = serde_json::from_str(msg).unwrap();
    match value["error"].as_object() {
        Some(obj) => {
            let error_message = obj.get("message").unwrap().as_str().unwrap().to_owned();
            error!(
                "extrinsic error code {}: {}",
                obj.get("code").unwrap().as_u64().unwrap(),
                error_message.clone()
            );
            (XtStatus::Error, Some(error_message))
        }
        None => match value["params"]["result"].as_object() {
            Some(obj) => {
                if let Some(hash) = obj.get("finalized") {
                    info!("finalized: {:?}", hash);
                    (XtStatus::Finalized, Some(hash.to_string()))
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
        assert_eq!(parse_status(msg), (XtStatus::Error, Some("Parse error".into())));

        let msg = "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":1010,\"message\":\"Invalid Transaction\",\"data\":0},\"id\":\"4\"}";
        assert_eq!(parse_status(msg), (XtStatus::Error, Some("Invalid Transaction".into())));

        let msg = "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":1001,\"message\":\"Extrinsic has invalid format.\"},\"id\":\"0\"}";
        assert_eq!(parse_status(msg), (XtStatus::Error, Some("Extrinsic has invalid format.".into())));

        let msg = r#"{"jsonrpc":"2.0","error":{"code":1002,"message":"Verification Error: Execution(Wasmi(Trap(Trap { kind: Unreachable })))","data":"RuntimeApi(\"Execution(Wasmi(Trap(Trap { kind: Unreachable })))\")"},"id":"3"}"#;
        assert_eq!(parse_status(msg), (XtStatus::Error, Some("Verification Error: Execution(Wasmi(Trap(Trap { kind: Unreachable })))".into())));
    }
}
