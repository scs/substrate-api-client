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

///! Very simple example that shows how to pretty print the metadata. Has proven to be a helpful
///! debugging tool.
use sp_core::sr25519;

use substrate_api_client::{rpc::WsRpcClient, Api, AssetTipExtrinsicParams, Metadata};

fn main() {
	env_logger::init();

	let client = WsRpcClient::new("ws://127.0.0.1:9944");
	let mut api = Api::<sr25519::Pair, _, AssetTipExtrinsicParams>::new(client).unwrap();

	let meta = api.metadata().clone();

	meta.print_overview();
	meta.print_pallets();
	meta.print_pallets_with_calls();
	meta.print_pallets_with_events();
	meta.print_pallets_with_errors();
	meta.print_pallets_with_constants();

	// Update the runtime and metadata.
	api.update_runtime().unwrap();

	// Print full substrate metadata json formatted.
	println!("{}", Metadata::pretty_format(&api.metadata().metadata).unwrap())
}
