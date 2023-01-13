# substrate-api-client

<p align="center">
<img src=./web3_foundation_grants_badge_black.svg width = 400>
</p>

substrate-api-client a library written in Rust for connecting to the substrate's RPC interface via WebSockets allowing to

* Compose extrinsics, send them and subscribe to updates (synchronously).
* supports `no_std` builds. Only the rpc-client is std only. For `no_std` builds, a custom rpc client needs to be implemented.
* Watch events and execute code upon events.
* Parse and print the node metadata.

## Prerequisites

In order to build the substrate-api-client and the examples, Rust and the wasm target are needed. For Linux:
```bash
curl https://sh.rustup.rs -sSf | sh
# Install the rust toolchain specified in rust-toolchain.toml
rustup show
```
###  Substrate node

To execute the examples, a running substrate node is needed. You can download a node artifact from substrate directly: https://github.com/paritytech/substrate
or run the kitchensink-node with docker:

```
docker run -p 9944:9944 -p 9933:9933 -p 30333:30333 parity/substrate:latest --dev --ws-external --rpc-external
```

For more information, please refer to the [substrate](https://github.com/paritytech/substrate) repository.

## Examples

To run an example, clone the `substrate-api-client` repository and run the desired example directly with the cargo command:

```bash
git clone https://github.com/scs/substrate-api-client.git
cd substrate-api-client
cargo run -p ac-examples --example get_storage
```
or download the already built binaries from [GitHub Actions](https://github.com/scs/substrate-api-client/actions) and run them without any previous building:

```bash
# Add execution rights to the chosen example.
chmod +x <example>
# And run it.
./<example>
```


Set the output verbosity by prepending `RUST_LOG=info` or `RUST_LOG=debug`.

The following examples can be found in the [examples](/examples/examples) folder:

 * [benchmark_bulk_xt](/examples/examples/benchmark_bulk_xt.rs): Float the node with a series of transactions.
* [compose_extrinsic_offline](/examples/examples/compose_extrinsic_offline.rs): Compose an extrinsic without interacting with the node.
* [custom_nonce](/examples/examples/custom_nonce.rs): Compose an with a custom nonce.
* [contract_instantiate_with_code](/examples/examples/contract_instantiate_with_code.rs): Instantiate a contract on the chain.
* [event_callback](/examples/examples/event_callback.rs): Subscribe and react on events.
* [event_error_details](/examples/examples/event_error_details.rs): Listen to error events from the node to determine if an extrinsic was successful or not.
* [get_account_identity](/examples/examples/get_account_identit.rs): Create an custom Unchecked Extrinsic to set an account identity and retrieve it afterwards with a getter.
* [get_block](/examples/examples/get_block.rs): Read header, block and signed block from storage.
* [get_storage](/examples/examples/get_storage.rs): Read storage values.
* [print_metadata](/examples/examples/print_metadata.rs): Print the metadata of the node in a readable way.
* [staking_batch_payout](/src/examples/examples/staking_batch_payout.rs): Batch reward payout for validator.
* [sudo](/examples/examples/sudo.rs): Create and send a sudo wrapped call.
* [transfer_with_tungstenite_client](/examples/examples/transfer_with_tungstenite_client.rs): Transfer tokens by using a wrapper of compose_extrinsic with an account generated with a seed.
* [transfer_with_ws_client](/examples/examples/transfer_with_ws_client.rs): Transfer tokens by using a wrapper of compose_extrinsic with an account generated with a seed.

## No_std Build
Everything, except for the [rpc-clients](https://github.com/scs/substrate-api-client/tree/master/src/rpc) is `no_std` compatible. Some selected features are also std-only.
Therefore, if `std` is available, it is recommended to use in std-mode.
Many features, such as extrinsic creation (see the [macros](https://github.com/scs/substrate-api-client/blob/master/compose-macros/src/lib.rs)), metadata and event types (see the [node-api](https://github.com/scs/substrate-api-client/tree/master/node-api/src) and [primitives](https://github.com/scs/substrate-api-client/tree/master/primitives/src)) are available right away. However, to directly connect to a Substrate node, a RPC client is necessary. In the following, it is explained how this may be achieved.

### Import
To import the api-client in `no_std` make sure the default features are turned off and `disable_target_static_assertions` is enabled:
```toml
# In the Cargo.toml import the api-client as following:
substrate-api-client = { git = "https://github.com/scs/substrate-api-client.git", default-features = false, features = ["disable_target_static_assertions"] }

```
### RPC Client
Depending on the usage, there are two traits that the Rpc Client may need to implemented. `Request` and `Subscribe`:
#### Request

For simple requests that (request and one answer) the trait [`Request`](https://github.com/scs/substrate-api-client/blob/d0a875e70f688c8ae2ce641935189c6374bc0ced/src/rpc/mod.rs#L44-L48) is used:
```rust
/// Trait to be implemented by the ws-client for sending rpc requests and extrinsic.
pub trait Request {
	/// Sends a RPC request to the substrate node and returns the answer as string.
	fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> Result<R>;
}
```
By implementing this trait with a custom RPC client, most basic functionalities of the `Api` can already be used.
Currently, there is no `no_std` example available that shows the full implementation of the `Request` trait. But the [`tungstenite_client`](https://github.com/scs/substrate-api-client/tree/d0a875e70f688c8ae2ce641935189c6374bc0ced/src/rpc/tungstenite_client) is a relatively simple `std` example:

```rust
impl Request for TungsteniteRpcClient {
	fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> Result<R> {
		let json_req = to_json_req(method, params)?;
		let response = self.direct_rpc_request(json_req)?;
		let deserialized_value: R = serde_json::from_str(&response)?;
		Ok(deserialized_value)
	}
}

impl TungsteniteRpcClient {
	fn direct_rpc_request(&self, json_req: String) -> Result<String> {
		let (mut socket, response) = attempt_connection_until(&self.url, self.max_attempts)?;
		debug!("Connected to the server. Response HTTP code: {}", response.status());

		// Send request to server.
		socket.write_message(Message::Text(json_req))?;

		let msg = read_until_text_message(&mut socket)?;

		debug!("Got get_request_msg {}", msg);
		let result_str =
			serde_json::from_str(msg.as_str()).map(|v: Value| v["result"].to_string())?;
		Ok(result_str)
	}
}
```
If a websocket library is available in your `no_std` environment, then your implementation may look similar.

#### Subscription
 A little more complex is the second trait [`Subscribe`](https://github.com/scs/substrate-api-client/blob/d0a875e70f688c8ae2ce641935189c6374bc0ced/src/rpc/mod.rs#L50-L62), which does not only send a subscription request to the node, it also keeps listening and updating accordingly.
Two traits need to be implemented for this feature.
The `Subscribe` trait itself:
```rust
/// Trait to be implemented by the ws-client for subscribing to the substrate node.
pub trait Subscribe {
	type Subscription<Notification>: HandleSubscription<Notification>
	where
		Notification: DeserializeOwned;

	fn subscribe<Notification: DeserializeOwned>(
		&self,
		sub: &str,
		params: RpcParams,
		unsub: &str,
	) -> Result<Self::Subscription<Notification>>;
}
```
and the [`HandleSubscription`](https://github.com/scs/substrate-api-client/blob/d0a875e70f688c8ae2ce641935189c6374bc0ced/src/rpc/mod.rs#L64-L78) trait, which is used by the returned `Subscription`:
```rust
/// Trait to use the full functionality of jsonrpseee Subscription type
/// without actually enforcing it.
pub trait HandleSubscription<Notification: DeserializeOwned> {
	/// Returns the next notification from the stream.
	/// This may return `None` if the subscription has been terminated,
	/// which may happen if the channel becomes full or is dropped.
	///
	/// **Note:** This has an identical signature to the [`StreamExt::next`]
	/// method (and delegates to that). Import [`StreamExt`] if you'd like
	/// access to other stream combinator methods.
	fn next(&mut self) -> Option<Result<Notification>>;

	/// Unsubscribe and consume the subscription.
	fn unsubscribe(self) -> Result<()>;
}
```
Refering to the `std` example of [tungstenite_client](https://github.com/scs/substrate-api-client/blob/d0a875e70f688c8ae2ce641935189c6374bc0ced/src/rpc/tungstenite_client/subscription.rs), the `HandleSubscription` impl could like the following. A simple channel receiver, waiting for the sender used by the ws-client to send something:
```rust
pub struct TungsteniteSubscriptionWrapper<Notification> {
	receiver: Receiver<String>,
	_phantom: PhantomData<Notification>,
}

impl<Notification> TungsteniteSubscriptionWrapper<Notification> {
	pub fn new(receiver: Receiver<String>) -> Self {
		Self { receiver, _phantom: Default::default() }
	}
}

impl<Notification: DeserializeOwned> HandleSubscription<Notification>
	for TungsteniteSubscriptionWrapper<Notification>
{
	fn next(&mut self) -> Option<Result<Notification>> {
		let notification = match self.receiver.recv() {
			Ok(notif) => notif,
			// Sender was disconnected, therefore no further messages are to be expected.
			Err(_e) => return None,
		};
		Some(serde_json::from_str(&notification).map_err(|e| e.into()))
	}

	fn unsubscribe(self) -> Result<()> {
		// We close ungracefully: Simply drop the receiver. This will turn
		// into an error on the sender side, terminating the websocket polling loop.
		Ok(())
	}
}
```
Now, if a websocket client is available in your `no_std` environment, the implementation of `Subscribe` could look like:
```rust
impl Subscribe for TungsteniteRpcClient {
	type Subscription<Notification> = TungsteniteSubscriptionWrapper<Notification> where Notification: DeserializeOwned;

	fn subscribe<Notification: DeserializeOwned>(
		&self,
		sub: &str,
		params: RpcParams,
		_unsub: &str,
	) -> Result<Self::Subscription<Notification>> {
		let json_req = to_json_req(sub, params)?;
		let (result_in, receiver) = channel();
		self.start_rpc_client_thread(json_req, result_in)?;
		let subscription = TungsteniteSubscriptionWrapper::new(receiver);
		Ok(subscription)
	}
}

impl TungsteniteRpcClient {
    fn start_rpc_client_thread(
		&self,
		json_req: String,
		result_in: ThreadOut<String>,
	) -> Result<()> {
		let url = self.url.clone();
		thread::spawn(move || {
			let (mut socket, response) = connect(url)?;
	debug!("Connected to the server. Response HTTP code: {}", response.status());

	// Subscribe to server
	socket.write_message(Message::Text(json_req))?;

	loop {
		let msg = read_until_text_message(&mut socket)?;
		send_message_to_client(result_in.clone(), msg.as_str())?;
	}
		});
		Ok(())
	}
}
```
Keep in mind, this is a very simple example, and not something that is future and error proof. A production ready example can be taken by the [jsonrpsee](https://github.com/paritytech/jsonrpsee) client implementation

## Alternatives

Parity offers a Rust client with similar functionality: https://github.com/paritytech/subxt

## Acknowledgements

The development of substrate-api-client is financed by [web3 foundation](https://web3.foundation/)'s grant programme.

We also thank the teams at

* [Parity Technologies](https://www.parity.io/) for building [substrate](https://github.com/paritytech/substrate) and supporting us during development.

## Projects using substrate-api-client
If you intend to or are using substrate-api-client, please add your project [here](https://github.com/scs/substrate-api-client/edit/master/README.md)

_In alphabetical order_

- [Ajuna Network](https://github.com/ajuna-network)
- [Encointer](https://github.com/encointer)
- [Integritee Network](https://github.com/integritee-network)
- [Litentry](https://github.com/litentry)
- [Polkadex](https://github.com/Polkadex-Substrate)
