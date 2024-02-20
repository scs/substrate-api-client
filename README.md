# substrate-api-client

The substrate-api-client is a Rust library for connecting to a [substrate](https://substrate.io/)-based node via RPC. It's particularly useful for setups with no-std environment (which are typical for trusted execution environmnets or embedded devices). It provides similar functionalities as [Polkadot-js](https://wiki.polkadot.network/docs/polkadotjs), such as easy extrinsic submission and state queries. With an RPC client, developers can easily interact with any [Polkadot](https://polkadot.network/) or [Kusama](https://kusama.network/) chain. There are several [RPC clients](https://wiki.polkadot.network/docs/build-tools-index#rpc-and-api-tools) available in different programming languages. For Rust, the most popular RPC client is [subxt](https://github.com/paritytech/subxt). The substrate-api-client provides a simpler, less extensive alternative to subxt, focused on providing as many features as possible for no-std environments.

The substrate-api-client connects to the substrate's RPC interface via WebSockets allowing to

* Compose extrinsics, send them (asynchronously and synchronously) and subscribe to updates (synchronously).
* Support `no_std` builds. Only the rpc-client is std only. For `no_std` builds, a custom rpc client needs to be implemented.
* Watch events and execute code upon events.
* Parse and print the node metadata.
* Support async and sync implementations.
* Support three different websocket crates (`jsonrpsee`, `tungstenite` and `ws`). See `Cargo.toml` for more information and limitations.

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
docker run -p 9944:9944 -p 9933:9933 -p 30333:30333 parity/substrate:latest --dev --rpc-external
```

For more information, please refer to the [substrate](https://github.com/paritytech/substrate) repository.

## Examples
The api-client provides several examples which show how to fetch node states or submit extrinsic. Examples are differentiated between `sync` and `async` implementations. Don't forget to check the feature import of the associated `Cargo.toml`. It shows how to import the api-client as an `async` or `sync` library.
To run an example, clone the `substrate-api-client` repository and run the desired example directly with the cargo command:

```bash
git clone https://github.com/scs/substrate-api-client.git
cd substrate-api-client
# Run an async example:
cargo run -p ac-examples-async --example get_storage
# Run a sync example:
cargo run -p ac-examples-sync --example runtime_update_sync
```

or download the already built binaries from [GitHub Actions](https://github.com/scs/substrate-api-client/actions) and run them without any previous building:

```bash
# Enter the async or sync example directory and add execution rights to the chosen example.
cd examples-<sync/async>
chmod +x <example>
# And run it.
./<example>
```


Set the output verbosity by prepending `RUST_LOG=info` or `RUST_LOG=debug`.

The following async examples can be found in the [async examples](/examples/async/examples) folder:
* [benchmark_bulk_xt](/examples/async/examples/benchmark_bulk_xt.rs): Float the node with a series of transactions.
* [check_extrinsic_events](/examples/async/examples/check_extrinsic_events.rs): Check and react according to events associated to an extrinsic.
* [compose_extrinsic](/examples/async/examples/compose_extrinsic.rs): Compose an extrinsic without interacting with the node or in no_std mode.
* [contract_instantiate_with_code](/examples/async/examples/contract_instantiate_with_code.rs): Instantiate a contract on the chain.
* [custom_nonce](/examples/async/examples/custom_nonce.rs): Compose an with a custom nonce.
* [get_account_identity](/examples/async/examples/get_account_identity.rs): Create an custom Unchecked Extrinsic to set an account identity and retrieve it afterwards with a getter.
* [get_blocks](/examples/async/examples/get_blocks.rs): Read header, block and signed block from storage.
* [get_storage](/examples/async/examples/get_storage.rs): Read storage values.
* [print_metadata](/examples/async/examples/print_metadata.rs): Print the metadata of the node in a readable way.
* [query_runtime_api](/src/examples/async/examples/query_runtime_api.rs): How to query the runtime api.
* [runtime_update_async](/examples/async/examples/runtime_update_async.rs): How to do an runtime upgrade asynchronously.
* [staking_batch_payout](/examples/async/examples/staking_batch_payout.rs): Batch reward payout for validator.
* [subscribe_events](/examples/async/examples/subscribe_events.rs): Subscribe and react on events.
* [sudo](/examples/async/examples/sudo.rs): Create and send a sudo wrapped call.

The following sync examples can be found in the [sync examples](/examples/sync/examples) folder:
* [runtime_update_sync](/examples/sync/examples/runtime_update_sync.rs): How to do an runtime upgrade synchronously.
* [transfer_with_tungstenite_client](/examples/sync/examples/transfer_with_tungstenite_client.rs): Transfer tokens by using a wrapper of compose_extrinsic with an account generated with a seed.
* [transfer_with_ws_client](/examples/sync/examples/transfer_with_ws_client.rs): Transfer tokens by using a wrapper of compose_extrinsic with an account generated with a seed.


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
Depending on the usage, there are two traits that the RPC Client needs to implement. You can choose between the sync and async implementation. If you decide to use the async implementation, you need to use the library `async-trait` for now (until it is integrated into the rust toolchain).

#### Request
For simple requests (send one request and receive one answer) the trait [`Request`](https://github.com/scs/substrate-api-client/blob/d0a875e70f688c8ae2ce641935189c6374bc0ced/src/rpc/mod.rs#L44-L48) is required:
```rust
/// Trait to be implemented by the ws-client for sending rpc requests and extrinsic.
pub trait Request {
	/// Sends a RPC request to the substrate node and returns the answer as string.
	(async) fn request<R: DeserializeOwned>(&self, method: &str, params: RpcParams) -> Result<R>;
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

	(async) fn subscribe<Notification: DeserializeOwned>(
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
	(async) fn next(&mut self) -> Option<Result<Notification>>;

	/// Unsubscribe and consume the subscription.
	(async) fn unsubscribe(self) -> Result<()>;
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

The development of the substrate-api-client has been financed by:

- [web3 foundation](https://github.com/w3f/General-Grants-Program/blob/master/grants/speculative/substrate-api-client.md)
- [Integritee](https://integritee.network/)
- Kusama Treasury:
  - [Maintenance Nov-22 to Jan-23](https://kusama.polkassembly.io/treasury/237)
  - [Maintenance Feb-23 to Apr-23](https://kusama.polkassembly.io/treasury/267)
  - [Maintenance May-23 to Jul-23](https://kusama.polkassembly.io/treasury/312)
- Polkadot Treasury:
  - [Maintenance Sep-23 to Dec-23](https://polkadot.polkassembly.io/referenda/118)

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


## FAQ
1. Q: Everything compiles but the Substrate node does not accept my extrinsic or returns an error even if the extrinsic should be correct.

    A: First, ensure the api-client and the Substrate node have a matching version. E.g. if the node is running on `release-polkadot-v1.2.0`, checkout and compile a matching branch of the api-client. We are using the same naming scheme as Parity does. Please note: Not all Polkadot releases are published for all api-client releases. Which Polkadot releases are supported by which api-client release are noted in the [release notes](https://github.com/scs/substrate-api-client/releases). Don't find the release-match you're looking for? Feel free to request it via an issue.

2. Q: I get the error `Bad input data provided to validate_transaction` from the node when submitting an extrinsic. Even though I made sure the api-client and Polkadot releases are matching.

    A: Every extrinsic contains some node specific data. The tips for example may be provided by the `Asset` pallet or, by default, by the `Balances` pallet. The current api-client does not have access to this information. Therefore, these config data must be configured manually. Currently, there are two pre-defined Runtime Configs which should match most of the Substrate nodes:
    - [Asset Runtime Config](https://github.com/scs/substrate-api-client/blob/master/primitives/src/config/asset_runtime_config.rs): This matches most nodes config that use the `Asset` pallet.
    - [Default Runtime Config](https://github.com/scs/substrate-api-client/blob/master/primitives/src/config/default_runtime_config.rs): This matches most of node runtimes that do not use the `Asset` pallet. The config, apart from the tip parameter, equals the [asset runtime config](https://github.com/scs/substrate-api-client/blob/master/primitives/src/config/asset_runtime_config.rs).

    Ensure you're using a matching config. If you do not use default parameters as configured in one of the provided configs, you must provide your own config that implements the [Config trait](https://github.com/scs/substrate-api-client/blob/master/primitives/src/config/mod.rs).

3. Q: I want to query a state from a substrate node via the api-client, but I do not get the expected value, respective the decoding fails. How come?

    A: When specifying your own state query, you must provide the return type of the state you're trying to retrieve. This is because the api-client only gets bytes from the node and must be able to deserialize these properly. That is not possible without knowing the type to decode to. This type may be for example a simple `u64` for retrieving the `Balance` of an account. But careful: If you're looking at the pallet code and its return type, don't forget to take the Query type into consideration. The `OptionQuery` for example automatically wraps the return type into an `Option` (see the [substrate docs "Handling query return values"](https://docs.substrate.io/build/runtime-storage/) for more information). Alternatively, you can always double check via [polkadot.js](https://polkadot.js.org/).
	If you're importing a value directly from the runtime, as it's done in this [example](https://github.com/scs/substrate-api-client/blob/fb108a7d1994705bbca50233e3bc66cec3726523/examples/examples/subscribe_events.rs#L25-L27), remember to adapt it to the node you are querying from.
