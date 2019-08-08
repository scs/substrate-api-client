# substrate-api-client
Library for connecting to substrate API over WebSockets

Composes Extrinsics, sends them and subscribes to updates.

Can watch events and execute code upon events.

## Setup

Run substrate node (examples use hardcoded url=localhost and ws-port=9944)

    substrate --dev

Run examples

    git clone https://github.com/scs/substrate-api-client.git
    cd substrate-api-client
    cargo run --example example-get-storage

Set the output verbosity by adding `RUST_LOG=info` or `RUST_LOG=debug` in front of the command.

## Reading storage

    extern crate substrate_api_client;
    use substrate_api_client::{Api, hexstr_to_u256};
    use keyring::AccountKeyring;
    use node_primitives::AccountId;
    use parity_codec::Encode;

    fn main() {
        let api = Api::new("ws://127.0.0.1:9944".to_string());

        // get some plain storage value
        let result_str = api.get_storage("Balances", "TransactionBaseFee", None).unwrap();
        let result = hexstr_to_u256(result_str);
        println!("[+] TransactionBaseFee is {}", result);

        // get Alice's AccountNonce
        let accountid = AccountId::from(AccountKeyring::Alice);
        let result_str = api.get_storage("System", "AccountNonce", Some(accountid.encode())).unwrap();
        let result = hexstr_to_u256(result_str);
        println!("[+] Alice's Account Nonce is {}", result);
    }

See [example_get_storage.rs](./src/bin/example_get_storage.rs)

## Sending transactions
See [example_transfer.rs](./src/bin/example_transfer.rs)

## Execute code upon events
See [example_event_callback.rs](./src/bin/example_event_callback.rs)

## TODO
  * dynamic API from metadata
  * compose custom runtime module extrinsics generically