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

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Serde json error: {0}")]
	Serde(#[from] serde_json::error::Error),
	#[error("Extrinsic Error: {0}")]
	Extrinsic(String),
	#[error("mpsc send Error: {0}")]
	Send(String),
	#[error("Could not convert hex value into a string: {0}")]
	Hex(#[from] hex::FromHexError),
	// Ws error generates clippy warnings without Box, see #303
	#[cfg(feature = "ws-client")]
	#[error("Websocket ws error: {0}")]
	Ws(Box<ws::Error>),
	#[cfg(feature = "tungstenite-client")]
	#[error("WebSocket tungstenite Error: {0}")]
	TungsteniteWebSocket(#[from] tungstenite::Error),
	#[error("Expected some error information, but nothing was found: {0}")]
	NoErrorInformationFound(String),
	#[error("ChannelReceiveError, sender is disconnected: {0}")]
	Disconnected(#[from] sp_std::sync::mpsc::RecvError),
	#[error("Failure during thread creation: {0}")]
	Io(#[from] std::io::Error),
	#[error(transparent)]
	Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
	#[error("Exceeded the maximum number of attempts to connect to the server")]
	ConnectionAttemptsExceeded,
}

#[cfg(feature = "ws-client")]
impl From<ws::Error> for Error {
	fn from(error: ws::Error) -> Self {
		Self::Ws(Box::new(error))
	}
}
