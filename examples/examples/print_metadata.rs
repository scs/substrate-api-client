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

//! Very simple example that shows how to pretty print the metadata. Has proven to be a helpful
//! debugging tool.

use substrate_api_client::{
	ac_primitives::AssetRuntimeConfig, api_client::UpdateRuntime, rpc::JsonrpseeClient, Api,
};

// To test this example with CI we run it against the Substrate kitchensink node, which uses the asset pallet.
// Therefore, we need to use the `AssetRuntimeConfig` in this example.
// ! However, most Substrate runtimes do not use the asset pallet at all. So if you run an example against your own node
// you most likely should use `DefaultRuntimeConfig` instead.

#[tokio::main]
async fn main() {
	env_logger::init();

	// Initialize the api, which retrieves the metadata from the node upon initialization.
	let client = JsonrpseeClient::with_default_url().unwrap();
	let mut api = Api::<AssetRuntimeConfig, _>::new(client).unwrap();

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
	println!("{}", (&api.metadata().pretty_format().unwrap()))
}
