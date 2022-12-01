# substrate-api-client

<p align="center">
<img src=./web3_foundation_grants_badge_black.svg width = 400>
</p>

substrate-api-client a library written in Rust for connecting to the substrate's RPC interface via WebSockets allowing to

* Compose extrinsics, send them and subscribe to updates (synchronously).
* supports composing extrinsics for `no_std` builds
* Watch events and execute code upon events.
* Parse and print the node metadata.

## Prerequisites

In order to build the substrate-api-client and the examples, Rust and the wasm target are needed. For Linux:
```bash
curl https://sh.rustup.rs -sSf | sh
# Install the rust toolchain specified in rust-toolchain.toml
rustup show
```
For more information, please refer to the [substrate](https://github.com/paritytech/substrate) repository.

## Substrate node

To execute the examples, a running substrate node is needed. You can download a node artifact from substrate directly: https://github.com/paritytech/substrate
or run the kitchensink-node with docker:

```
docker run -p 9944:9944 -p 9933:9933 -p 30333:30333 parity/substrate:latest --dev --ws-external --rpc-external
```

## Examples

To run an example, clone the `substrate-api-client` repository and run the desired example directly with the cargo command:

```bash
git clone https://github.com/scs/substrate-api-client.git
cd substrate-api-client
cargo run --example get_storage
```
or download the already built binaries from Github Actions: https://github.com/scs/substrate-api-client/actions and run them without any building:

```bash
chmod +x <example>
./<example>
```


Set the output verbosity by prepending `RUST_LOG=info` or `RUST_LOG=debug`.

The following examples can be found in the [examples](/examples) folder:

* [benchmark_bulk_xt](/examples/benchmark_bulk_xt.rs): Float the node with a series of transactions.
* [compose_extrinsic_offline](/examples/compose_extrinsic_offline.rs): Compose an extrinsic without interacting with the node.
* [contract_instantiate_with_code](/examples/contract_instantiate_with_code.rs): Instantiate a contract on the chain.
* [event_callback](/examples/event_callback.rs): Subscribe and react on events.
* [event_error_details](/examples/event_error_details.rs): Listen to error events from the node to determine if an extrinsic was successful or not.
* [generic_event_callback](/examples/generic_event_callback.rs): Listen to an example event from the node.
* [generic_extrinsic](/examples/generic_extrinsic.rs): Compose an extrinsic for any call in any module by supplying the module and call name as strings.
* [get_block](/examples/get_block.rs): Read header, block and signed block from storage.
* [get_storage](/examples/get_storage.rs): Read storage values.
* [print_metadata](/examples/print_metadata.rs): Print the metadata of the node in a readable way.
* [sudo](/examples/sudo.rs): Create and send a sudo wrapped call.
* [transfer_using_seed](/examples/transfer_using_seed.rs): Transfer tokens by using a wrapper of compose_extrinsic with an account generated with a seed.
* [staking_payout](/src/examples/staking_payout.rs): Westend staking reward payout for validator.
* [batch_payout](/src/examples/staking_payout.rs): Batch reward payout for validator.

## Alternatives

Parity offers a Rust client with similar functionality: https://github.com/paritytech/substrate-subxt

## Acknowledgements

The development of substrate-api-client is financed by [web3 foundation](https://web3.foundation/)'s grant programme.

We also thank the teams at

* [Parity Technologies](https://www.parity.io/) for building [substrate](https://github.com/paritytech/substrate) and supporting us during development.

## Projects using substrate-api-client
- [If you intend to or are using substrate-api-client, please add your project here](https://github.com/scs/substrate-api-client/edit/master/README.md)

_In alphabetical order_

- [Encointer](https://github.com/encointer)
- [Integritee Network](https://github.com/integritee-network)
