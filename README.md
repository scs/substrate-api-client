# substrate-api-client

<p align="center">
<img src=./web3_foundation_grants_badge_black.svg width = 400>
</p>

substrate-api-client a library written in Rust for connecting to the substrate's RPC interface via WebSockets allowing to

* Compose extrinsics, send them and subscribe to updates (synchronously).
* supports`no_std` builds. Only the rpc-client is std only. For `no_std` builds, a custom rpc client needs to be implemented.
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
cargo run -p ac-examples --example get_storage
```
or download the already built binaries from Github Actions: https://github.com/scs/substrate-api-client/actions and run them without any building:

```bash
chmod +x <example>
./<example>
```


Set the output verbosity by prepending `RUST_LOG=info` or `RUST_LOG=debug`.

The following examples can be found in the [examples](/examples/examples) folder:

* [staking_batch_payout](/src/examples/examples/staking_batch_payout.rs): Batch reward payout for validator.
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
* [sudo](/examples/examples/sudo.rs): Create and send a sudo wrapped call.
* [transfer_with_tungstenite_client](/examples/examples/transfer_with_tungstenite_client.rs): Transfer tokens by using a wrapper of compose_extrinsic with an account generated with a seed.
* [transfer_with_ws_client](/examples/examples/transfer_with_ws_client.rs): Transfer tokens by using a wrapper of compose_extrinsic with an account generated with a seed.

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
