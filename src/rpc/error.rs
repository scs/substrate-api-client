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

use std::sync::mpsc::SendError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Serde json error: {0}")]
	Serde(#[from] serde_json::error::Error),
	#[error("mpsc send Error: {0}")]
	Send(String),
	#[error("Could not convert to valid Url: {0}")]
	Url(#[from] url::ParseError),
	#[error("ChannelReceiveError, sender is disconnected: {0}")]
	ChannelDisconnected(#[from] sp_std::sync::mpsc::RecvError),
	#[error("Failure during thread creation: {0}")]
	Io(#[from] std::io::Error),
	#[error("Exceeded maximum amount of connections")]
	ConnectionAttemptsExceeded,
	#[error("Websocket Connection was closed unexpectedly")]
	ConnectionClosed,
	#[error(transparent)]
	Client(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[cfg(feature = "ws-client")]
impl From<SendError<String>> for Error {
	fn from(error: SendError<String>) -> Self {
		Self::Send(error.0)
	}
}

#[cfg(feature = "ws-client")]
impl From<ws::Error> for Error {
	fn from(error: ws::Error) -> Self {
		Self::Client(Box::new(error))
	}
}

#[cfg(feature = "tungstenite-client")]
impl From<tungstenite::Error> for Error {
	fn from(error: tungstenite::Error) -> Self {
		Self::Client(Box::new(error))
	}
}
