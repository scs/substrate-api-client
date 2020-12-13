# substrate-api-client

<p align="center">
<img src=./web3_foundation_grants_badge_black.svg width = 400>
</p>

substrate-api-client a library written in Rust for connecting to the substrate's RPC interface via WebSockets allowing to

* Compose extrinsics, send them and subscribe to updates (synchronously).
* supports composing extrinsics for `no_std` builds
* Watch events and execute code upon events.
* Parse and print the node metadata.
* currently only supports [node-template](https://github.com/substrate-developer-hub/substrate-node-template) based chains (doesn't support the Account type of Polkadot/Kusama without patches)

## Prerequisites

In order to build the substrate-api-client and the examples, Rust and the wasm target are needed. For Linux:

    curl https://sh.rustup.rs -sSf | sh

    rustup default nightly
    rustup target add wasm32-unknown-unknown --toolchain nightly
    cargo install --git https://github.com/alexcrichton/wasm-gc

For more information, please refer to the [substrate](https://github.com/paritytech/substrate) repository.

## Substrate node

To execute the examples, a running substrate node is needed. A slightly extended node-template can be found at https://github.com/scs/substrate-test-nodes.


To build the test node, execute the following steps:

    git clone https://github.com/scs/substrate-node-template
    cd substrate-test-nodes/
    cargo build --release

Run the node:

    ./target/release/substrate-test-node --dev

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

* [example_compose_extrinsic_offline](/src/examples/example_compose_extrinsic_offline.rs): Compose an extrinsic without interacting with the node.
* [example_contract](/src/examples/example_contract.rs): Handle ink! contracts (put, create, and call).
* [example_custom_storage_struct](/src/examples/example_custom_storage_struct.rs): Fetch and decode custom structs from the runtime.
* [example_event_callback](/src/examples/example_event_callback.rs): Subscribe and react on events.
* [example_generic_extrinsic](/src/examples/example_generic_extrinsic.rs): Compose an extrinsic for any call in any module by supplying the module and call name as strings.
* [example_get_storage](/src/examples/example_get_storage.rs): Read storage values.
* [example_print_metadata](/src/examples/example_print_metadata.rs): Print the metadata of the node in a readable way.
* [example_transfer](/src/examples/example_transfer.rs): Transfer tokens by using a wrapper of compose_extrinsic

### ink! contract

Shows how to setup an ink! contract with the predefined contract extrinsics:

* put_code: Stores a contract wasm blob on the chain
* create: Creates an instance of the contract
* call: Calls a contract.

*Note*: This example only works with the substrate-test-node found in https://github.com/scs/substrate-test-nodes as the contract module is not included by default in a substrate node.

### Read custom storage struct

Shows how to fetch and decode a custom storage struct.

    // The custom struct that is to be decoded. The user must know the structure for this to work, which can fortunately
    // be looked up from the node metadata and printed with the `example_print_metadata`.
    #[derive(Encode, Decode, Debug)]
    struct Kitty {
        id: H256,
        price: u128,
    }

...

    // Get the Kitty
    let res_str = api.get_storage("Kitty",
                                  "Kitties",
                                  Some(index.encode())).unwrap();

    let res_vec = Vec::from_hex(res_str);

    // Type annotations are needed here to know that to decode into.
    let kitty: Kitty = Decode::decode(&mut res_vec.as_slice()).unwrap();
    println!("[+] Cute decoded Kitty: {:?}\n", kitty);

*Note*: This example only works with the substrate-test-node found in https://github.com/scs/substrate-test-nodes for obvious reasons.

### Sending generic extrinsics

Shows how to use the compose_extrinsic! macro that is able to create an extrinsic for any kind of call, even for custom runtime modules.

    // Exchange "Balance" and "transfer" with the names of your custom runtime module. They are only
    // used here to be able to run the examples against a generic substrate node with standard modules.
    let xt: UncheckedExtrinsicV3<_, sr25519::Pair>  = compose_extrinsic!(
        api.clone(),
        "Balances",
        "transfer",
        GenericAddress::from(to.0.clone()),
        Compact(42 as u128)
    );

### Reading storage

Shows how to read some storage values.

    // get some plain storage value
    let result_str = api.get_storage("Balances", "TotalIssuance", None).unwrap();
    let result = hexstr_to_u256(result_str).unwrap();
    println!("[+] TotalIssuance is {}", result);

    // get Alice's AccountNonce
    let accountid = AccountId::from(AccountKeyring::Alice);
    let result_str = api
        .get_storage("System", "AccountNonce", Some(accountid.encode()))
        .unwrap();
    let result = hexstr_to_u256(result_str).unwrap();
    println!("[+] Alice's Account Nonce is {}", result.low_u32());

    // get Alice's AccountNonce with the AccountKey
    let signer = AccountKeyring::Alice.pair();
    let result_str = api
        .get_storage("System", "AccountNonce", Some(signer.public().encode()))
        .unwrap();
    let result = hexstr_to_u256(result_str).unwrap();
    println!("[+] Alice's Account Nonce is {}", result.low_u32());

    // get Alice's AccountNonce with api.get_nonce()
    api.signer = Some(signer);
    println!("[+] Alice's Account Nonce is {}", api.get_nonce().unwrap());

## Alternatives

Parity offers a Rust client with similar functionality: https://github.com/paritytech/substrate-subxt

## Acknowledgements

The development of substrate-api-client is financed by [web3 foundation](https://web3.foundation/)'s grant programme.

We also thank the teams at

* [Parity Technologies](https://www.parity.io/) for building [substrate](https://github.com/paritytech/substrate) and supporting us during development.
