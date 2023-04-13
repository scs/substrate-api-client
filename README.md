# substrate-api-client

<p align="center">
<img src=./web3_foundation_grants_badge_black.svg width = 400>
</p>

substrate-api-client is a library written in Rust for connecting to the substrate's RPC interface via WebSockets allowing to

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

## `no_std` build
Almost everything in the api-client, except for the [rpc-clients](https://github.com/scs/substrate-api-client/tree/master/src/rpc) and a few additional features, is `no_std` compatible.
Many helpful features, such as extrinsic and call creation (see the [macros](https://github.com/scs/substrate-api-client/blob/master/compose-macros/src/lib.rs)), metadata and event types (see the [node-api](https://github.com/scs/substrate-api-client/tree/master/node-api/src) and [primitives](https://github.com/scs/substrate-api-client/tree/master/primitives/src)) are available in `no_std` right away. However, to directly connect to a Substrate node a RPC client is necessary. Because websocket connection features are often hardware dependent, a generic `no_std` RPC client implementation is hardly possible. So for most use cases a self-implemented RPC client is required. To make this as simple as possible, the interface between the `Api`, which provides all the features, and the RPC client, providing the node connection, is kept very basic. Check out the following explanations for more info.

### Import
To import the api-client in `no_std` make sure the default features are turned off and `disable_target_static_assertions` is enabled:
```toml
# In the Cargo.toml import the api-client as following:
substrate-api-client = { git = "https://github.com/scs/substrate-api-client.git", default-features = false, features = ["disable_target_static_assertions"] }

```
### RPC Client
Depending on the usage, there are two traits that the RPC Client needs to implement.

#### Request

For simple requests (send one request and receive one answer) the trait [`Request`](https://github.com/scs/substrate-api-client/blob/d0a875e70f688c8ae2ce641935189c6374bc0ced/src/rpc/mod.rs#L44-L48) is required:
```rust
/// Trait to be implemented by the ws-client for sending rpc requests and extrinsic.
pub trait Request {
	/// Sends a RPC request to the substrate node and returns the answer as string.
	fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> Result<R>;
}
```
By implementing this trait with a custom RPC client, most basic functionalities of the `Api` can already be used.
Currently, there is no `no_std` example available. But the [`tungstenite_client`](https://github.com/scs/substrate-api-client/blob/d0a875e70f688c8ae2ce641935189c6374bc0ced/src/rpc/tungstenite_client/client.rs#L41-L64) provides a relatively simple `std` example. If a websocket library is available in your `no_std` environment, then your implementation may look similar.

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
and the [`HandleSubscription`](https://github.com/scs/substrate-api-client/blob/d0a875e70f688c8ae2ce641935189c6374bc0ced/src/rpc/mod.rs#L64-L78) trait, which is returned by the `subscribe` function:
```rust
/// Trait to use the full functionality of jsonrpseee Subscription type
/// without actually enforcing it.
pub trait HandleSubscription<Notification: DeserializeOwned> {
	/// Returns the next notification from the stream.
	/// This may return `None` if the subscription has been terminated,
	/// which may happen if the channel becomes full or is dropped.
	fn next(&mut self) -> Option<Result<Notification>>;

	/// Unsubscribe and consume the subscription.
	fn unsubscribe(self) -> Result<()>;
}
```
Refering to the `std` example of the tungstenite, the `HandleSubscription` impl can be looked up [here](https://github.com/scs/substrate-api-client/blob/d0a875e70f688c8ae2ce641935189c6374bc0ced/src/rpc/tungstenite_client/subscription.rs#L23-L54). It implements a simple channel receiver, waiting for the sender of the websocket client to send something.
The `Subscribe` implementation can be found [here](https://github.com/scs/substrate-api-client/blob/d0a875e70f688c8ae2ce641935189c6374bc0ced/src/rpc/tungstenite_client/client.rs#L66-L81).

A more complex RPC client, but also with more functionalities, is the [jsonrpsee](https://github.com/paritytech/jsonrpsee) client.

## Example Upgrades from older to newer versions
There have been some breaking API changes as of late to catch up with the newer Substrate versions and to fully support different Substrate nodes.
An example project on how to upgrade from older tags can be found in the Integritee [worker repository](https://github.com/integritee-network/worker):
-  [tag v0.7.0 -> v0.9.0](https://github.com/integritee-network/worker/pull/1263) (Upgrade to tag v0.8.0 is not recommended, directly upgrading to v0.9.0 saves you some extra work).
- [tag v0.9.0 -> v0.10.0](https://github.com/integritee-network/worker/pull/1265)

If you still experience issues during upgrading, do not hesitate to create an issue for support.


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
