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

    curl https://sh.rustup.rs -sSf | sh

    rustup default nightly
    rustup target add wasm32-unknown-unknown --toolchain nightly

For more information, please refer to the [substrate](https://github.com/paritytech/substrate) repository.

## Substrate node

To execute the examples, a running substrate node is needed. You can download our node artifact from our GitHub Actions
CI, which can be found in the 'Actions' tab.

The unpacked node can be run with:

```
./node-template --dev
```


## Tutorial

There is a detailed tutorial in the [tutorials](/tutorials) folder.

## Examples

To run an example, clone the `substrate-api-client` repository and run the desired example directly with the cargo command:

```bash
    git clone https://github.com/scs/substrate-api-client.git
    cd substrate-api-client
    cargo run --example example_get_storage
```

Set the output verbosity by prepending `RUST_LOG=info` or `RUST_LOG=debug`.

The following examples can be found in the [examples](/src/examples) folder:

* [compose_extrinsic_offline](/src/examples/compose_extrinsic_offline.rs): Compose an extrinsic without interacting with the node.
* [contract](/src/examples/contract.rs): Handle ink! contracts (put, create, and call). **DEPRECATED!**
* [custom_storage_struct](/src/examples/custom_storage_struct.rs): Fetch and decode custom structs from the runtime. **DEPRECATED!**
* [event_callback](/src/examples/event_callback.rs): Subscribe and react on events.
* [generic_extrinsic](/src/examples/generic_extrinsic.rs): Compose an extrinsic for any call in any module by supplying the module and call name as strings.
* [get_storage](/src/examples/get_storage.rs): Read storage values.
* [print_metadata](/src/examples/print_metadata.rs): Print the metadata of the node in a readable way.
* [transfer](/src/examples/transfer.rs): Transfer tokens by using a wrapper of compose_extrinsic

## Alternatives

Parity offers a Rust client with similar functionality: https://github.com/paritytech/substrate-subxt

## Acknowledgements

The development of substrate-api-client is financed by [web3 foundation](https://web3.foundation/)'s grant programme.

We also thank the teams at

* [Parity Technologies](https://www.parity.io/) for building [substrate](https://github.com/paritytech/substrate) and supporting us during development.
