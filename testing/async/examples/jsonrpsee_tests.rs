/*
	Copyright 2024 Supercomputing Systems AG
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

//! Tests for the Jsonrpseeclient Wrapper. Should happen during runtime, otherwise no connection can be etablished.

use core::panic;

use jsonrpsee::{
	client_transport::ws::{Url, WsTransportClientBuilder},
	core::client::{Client, ClientBuilder},
	server::{RpcModule, Server},
};
use substrate_api_client::rpc::JsonrpseeClient;
use tokio::{sync::oneshot, task, time, time::Duration};

#[tokio::main]
async fn main() {
	let port = 9944;
	let address = "ws://127.0.0.1";

	let client = JsonrpseeClient::with_default_url().await;
	let client2 = JsonrpseeClient::new_with_port(address, port).await;
	let client3 = JsonrpseeClient::new_with_port(address, 9994).await;

	assert!(client.is_ok());
	assert!(client2.is_ok());
	// No node running at this port - creation should fail.
	assert!(client3.is_err());

	// Test new_with_client and extra functionality of inner Jsonrpseee client.
	let (tx, mut rx) = oneshot::channel();
	let finish_message = "Finishing task";

	// Start server.
	let server = Server::builder().build("127.0.0.1:0").await.unwrap();
	let addr = server.local_addr().unwrap();
	let server_handle = server.start(RpcModule::new(()));

	// Create client and connect.
	let uri = Url::parse(&format!("ws://{}", addr)).unwrap();
	let (tx1, rx1) = WsTransportClientBuilder::default().build(uri).await.unwrap();
	let client: Client = ClientBuilder::default().build_with_tokio(tx1, rx1);
	let api_rpsee_client = JsonrpseeClient::new_with_client(client);
	assert!(api_rpsee_client.is_connected());

	let client_handle = task::spawn(async move {
		println!("Waiting for client disconnect");
		api_rpsee_client.on_disconnect().await;
		time::sleep(Duration::from_secs(2)).await;
		println!("Disconnected due to: {:?}", api_rpsee_client.disconnect_reason().await);
		tx.send(finish_message).unwrap();
	});

	// Drop server such that client gets a disconnect.
	drop(server_handle);

	// Wait for the disconnect message.
	let timeout = 5;
	let mut ctr = 0;
	loop {
		if let Ok(message) = rx.try_recv() {
			assert_eq!(finish_message, message);
			println!("{message}");
			break;
		} else {
			ctr += 1;
			if ctr == timeout {
				panic!("Timeout");
			}
			time::sleep(Duration::from_secs(1)).await;
			println!("sleeping..");
		}
	}

	client_handle.await.unwrap();
}
