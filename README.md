# substrate-api-client
Library for connecting to the substrate's RPC interface via WebSockets allowing to

* Compose extrinsics, send them and subscribe to updates.
* Watch events and execute code upon events.
* Parse and print the note metadata.

## Prerequisites
In order to build the substrate-api-client and the examples, Rust and the wasm target are needed. For Linux:

    curl https://sh.rustup.rs -sSf | sh

    rustup default nightly
    rustup target add wasm32-unknown-unknown --toolchain nightly
    cargo install --git https://github.com/alexcrichton/wasm-gc

For more information, please refer to the [substrate](https://github.com/paritytech/substrate) repository.

## Setup

Setup a substrate node. Tested with [revision 9b08e7ff of substrate](https://github.com/paritytech/substrate/commit/9b08e7ff938a45dbec7fcdb854063202e2b0cb48). Alternatively, the test node found at https://github.com/scs/substrate-test-nodes is guaranteed to work, which is anyhow needed for some examples.

    git clone https://github.com/scs/substrate-test-nodes
    cd substrate-test-nodes/
    git checkout api-M1.1
    // --release flag needed as block production time is too long otherwise.
    cargo build --release
    
Run the node (examples use by default `url=localhost` and `ws-port=9944`):    
   
    ./target/release/substrate-test-node --dev
    
Run the examples (optionally adjust url and port if wanted, not needed if the node is run with default arguments)

    git clone https://github.com/scs/substrate-api-client.git
    cd substrate-api-client
    cargo run --example example_get_storage (-ns <custom url> -p <custom port>)

Set the output verbosity by prepending `RUST_LOG=info` or `RUST_LOG=debug`.

## Examples
To run an example, you can use i.e.
```
cargo run --example example_transfer -- --ns 192.168.1.4 --node-port 9944
```


### Reading storage
Shows how to read some storage values.

    // get some plain storage value
    let result_str = api.get_storage("Balances", "TransactionBaseFee", None).unwrap();
    let result = hexstr_to_u256(result_str);
    println!("[+] TransactionBaseFee is {}", result);

    // get Alice's AccountNonce
    let accountid = AccountId::from(AccountKeyring::Alice);
    let result_str = api.get_storage("System", "AccountNonce", Some(accountid.encode())).unwrap();
    let result = hexstr_to_u256(result_str);
    println!("[+] Alice's Account Nonce is {}", result.low_u32());

    // get Alice's AccountNonce with api.get_nonce()
    api.signer = Some(AccountKey::new("//Alice", Some(""), CryptoKind::Sr25519));
    println!("[+] Alice's Account Nonce is {}", api.get_nonce());


See [example_get_storage.rs](/src/examples/example_get_storage.rs)

### Sending transactions
Shows how to use one of the predefined extrinsics to send transactions.

See [example_transfer.rs](/src/examples/example_transfer.rs)

### Sending generic extrinsics
Shows how to use the compose_extrinsic! macro that is able to create an extrinsic for any kind of call, even for custom runtime modules.

    // Exchange "Balance" and "transfer" with the names of your custom runtime module. They are only
    // used here to be able to run the examples against a generic substrate node with standard modules.
    let xt = compose_extrinsic!(
        api.clone(),
        "Balances",
        "transfer",
        GenericAddress::from(to),
        Compact(42 as u128)
    );


See [example_generic_extrinsic.rs](/src/examples/example_generic_extrinsic.rs)

### Callback
Shows how to listen to events fired by a substrate node.

See [example_event_callback.rs](/src/examples/example_event_callback.rs)

### Pretty print metadata
Shows how to print a substrate node's metadata in pretty json format. Has been proven a useful debugging tool.

    let api = Api::new(format!("ws://{}", url));

    let meta = api.get_metadata();
    println!("Metadata:\n {}", node_metadata::pretty_format(&meta).unwrap());

See [example_print_metadata.rs](/src/examples/example_print_metadata.rs)

### ink! contract
Shows how to setup an ink! contract with the predefined contract extrinsics:
* put_code: Stores a contract wasm blob on the chain
* create: Create an instance of the contract
* call: Calls a contract.

See [example_contract.rs](/src/examples/example_contract.rs)

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

    let res_vec = hexstr_to_vec(res_str);

    // Type annotations are needed here to know that to decode into.
    let kitty: Kitty = Decode::decode(&mut res_vec.as_slice()).unwrap();
    println!("[+] Cute decoded Kitty: {:?}\n", kitty);

See [example_custom_storage_struct.rs](/src/examples/example_custom_storage_struct.rs)

*Note*: This example only works with the substrate-test-node found in https://github.com/scs/substrate-test-nodes for obvious reasons.

## Alternatives

Parity offers a Rust client with similar functionality: https://github.com/paritytech/substrate-subxt

## Acknowledgements

The development of substrate-api-client is financed by [web3 foundation](https://web3.foundation/)'s grant programme.

We also thank the teams at

* [Parity Technologies](https://www.parity.io/) for building [substrate](https://github.com/paritytech/substrate) and supporting us during development.
