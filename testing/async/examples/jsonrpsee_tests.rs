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

use substrate_api_client::rpc::JsonrpseeClient;

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
}
