# substrate-api-client tutorial

This little guide elaborates on the usage of the client we wrote to interact with [substrate](https://github.com/paritytech/substrate) based blockchains. We will show from scratch how to setup interaction with a substrate blockchain.

First, we need to get a node running. We will use a custom substrate node found in our [substrate-test-nodes](https://github.com/scs/substrate-test-nodes) repository.

- Download and build the node:
    ```bash
    git clone https://github.com/scs/substrate-test-nodes
    cd substrate-test-nodes/
    git checkout api-M1.1
    cargo build --release
    ```

- Run the node. The `dev` flag allows to run the node on its own without performing consensus with other nodes.
    ```bash
    ./target/release/substrate-test-node --dev
    ```

Now we will start to write client code that interacts with the node. The test-node includes a minimalistic Kitty runtime module derived from the [Substrate Collectables Workshop](https://substrate.dev/substrate-collectables-workshop/#/). Let's start from scratch by making a new rust project called `api-client-tutorial`.

```bash
    cd $HOME
    cargo new api-client-tutorial
```

The `Cargo.toml` has to have one sole dependency and should look like:
```
[package]
name = "api-client-tutorial"
version = "0.1.0"
authors = ["Supercomputing Systems AG <info@scs.ch>"]
edition = "2018"

[dependencies]
substrate-api-client = { git = "https://github.com/scs/substrate-api-client.git" }
```

If we don't now exactly what our blockchain node features or what the runtime module is called we want to interact with, we can query the node metadata with our client. In the `src/main.rs` we will do two things:
- First, we instantiate an Api that connects to a given url.
- Second, we query the node metadata with the `api.get_metadata()` and print it in pretty json format afterwards.

`src/main.rs:`
```rust
use substrate_api_client::{Api, node_metadata};

fn main() {
    // instantiate an Api that connects to the given address
    let url = "127.0.0.1:9944";
    // if no signer is set in the whole program, we need to give to Api a specific type instead of an associated type
    // as during compilation the type needs to be defined.
    let api = Api::<sr25519::Pair>::new(format!("ws://{}", url));

    let meta = api.get_metadata();
    println!("Metadata:\n {}", node_metadata::pretty_format(&meta).unwrap());
}
```

If we now run the binary with `cargo run`, the metadata is printed to the terminal. The following exempt will be found along the metadata, which tells us that there is a `KittyModule` followed by 
* `storage`: data that is stored on chain
* `calls`: runtime functions that can be called from the outside and 
* `events`: callbacks that are fired from the runtime that a client can subscribe to


```
    ...
    {
     "name": "KittyModule",
     "storage": {
      "prefix": "Kitty",
      "entries": [
       {
        "name": "Kitties",
        "modifier": "Default",
        "ty": {
         "Map": {
          "hasher": "Blake2_256",
          "key": "u64",
          "value": "Kitty<T::Hash, T::Balance>",
          "is_linked": false
         }
        },
        "default": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
         0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        "documentation": []
       },
       {
        "name": "KittyCount",
        "modifier": "Default",
        "ty": {
         "Plain": "u64"
        },
        "default": [0,0,0,0,0,0,0,0],
        "documentation": []
       },
       {
        "name": "KittyIndex",
        "modifier": "Default",
        "ty": {
         "Map": {
          "hasher": "Blake2_256",
          "key": "T::AccountId",
          "value": "u64",
          "is_linked": false
         }
        },
        "default": [0,0,0,0,0,0,0,0],
        "documentation": []
       }
      ]
     },
     "calls": [
      {
       "name": "create_kitty",
       "arguments": [
        {
         "name": "price",
         "ty": "T::Balance"
        }
       ],
       "documentation": []
      },
      {
       "name": "update_kitty",
       "arguments": [
        {
         "name": "price",
         "ty": "T::Balance"
        }
       ],
       "documentation": []
      }
     ],
     "event": [
      {
       "name": "StoredKitty",
       "arguments": [
        "AccountId",
        "u64"
       ],
       "documentation": []
      },
      {
       "name": "UpdatedKitty",
       "arguments": [
        "AccountId",
        "u64"
       ],
       "documentation": []
      }
     ],
     "constants": []
    }
    ...
```

As we have just created our node, no `Kitty` is stored on our substrate blockchain yet. But we find in the calls that there is a call named `create_kitty`, which presumably creates a `Kitty` with a price supplied as argument. We will call that function in order to create a `Kitty`. Calling a runtime function in substrate is done via an extrinsic, which is somewhat like a transaction in general blockchain jargon but not exactly, see [Extrinsics](https://substrate.dev/docs/en/overview/extrinsics). An extrinsic is always signed by the account that submits the extrinsic. Therefore, we set the `signer` field of the `Api` with an account, that is then used to sign the extrinsic.

```rust
let signer = AccountKeyring::Alice.pair();

let api = Api::new(format!("ws://{}", url)).set_signer(signer);
```

`AccountKeyring` belongs to the substrate crate `keyring` that offers some predefined keys facilitating smooth developer experience. Now we are ready to create an extrinsic for our `KittyModule`, which is performed via the `compose_extrinsic!` macro.

```rust
let xt: UncheckedExtrinsicV3<_, sr25519::Pair> = compose_extrinsic!(
    api.clone(),
    "KittyModule",
    "create_kitty",
    10 as u128
);
```
The first three arguments are always the `Api`, the runtime module name, and then the function name as defined in the metadata. Subsequently, the arguments of the runtime function that is called are supplied. Taking a look at the metadata, we see that the `create_kitty` call takes one argument, which is `price` of type `T::Balance`, which is a type alias defined in substrate for a `u128`. Therefore, we are free to use `u128` as both encode to the same bytes. The macro does then query the account nonce of the sender from the node and creates a signed extrinsic ready to be encoded and sent. We have to explicitly put a type annotation for the UncheckedExtrinsic as `ed25519` could also be used and the macro is not able to infer the type as macro expansion happens before names are resolved and types are inferred.

Note: The signing process is not straight forward. Additional information is included in the signing payload which is more than the extrinsic payload that only consists of the prepared call statements. All the details can be found in the code.

The following call submits an extrinsic to the node and waits for the transaction hash that is returned upon block inclusion of the extrinsic.

```rust
let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
```

Having received the transaction hash, we can check if the `Kitty` belonging to Alice's account has successfully been created. Again, looking at the metadata unveils that the there are two storage maps that are of interest to us. there is `KittyIndex`, which maps an `AccountId` to an index (`u64`) and there is `Kitties`, which maps this index to a `Kitty`. This indirect approach is overall computationally more efficient than a mapping from `AccountId` to `Kitty` directly. Thus, we need to lookup the index before we can access our `Kitty`.

We can query the storage values of a runtime module via the `Api`'s `get_storage` method.

```rust
let res_str = api.get_storage("Kitty",
                              "KittyIndex",
                              Some(signer.public().encode())).unwrap();
```

The first argument of the `get_storage` call is always the storage `Prefix` followed by the entry name. If the storage value is a map, the key needs to be suplied as argument. The RPC interface of substrate does return a hex encoded `string`. In the `substrate_api_client::util` module reside several functions to handle those values. The following fits our needs:

```rust
let index = hexstr_to_u64(res_str).unwrap();
```

Now that we now the index of our `Kitty` in the `Kitties` map we can finally have a look at our `Kitty`.

```rust
let res_str = api.get_storage("Kitty",
                              "Kitties",
                              Some(index.encode())).unwrap();

let res_vec = Vec::from_hex(res_str).unwrap();
```

Naturally, the `utils` module does not have a `hexstr_to_kitty` function, instead we can transform it into a byte vector. Now we must decode this vector into a `Kitty`, but neither Rust nor substrate does know the structure of our `Kitty`. But luckily, the metadata does! In the metadata we find that our `Kitty` looks as follows: `Kitty<T::Hash, T::Balance>`. Hence, we can define the structure on the client side and tell rust what to decode into. `T::Hash` is again a substrate type alias for a 32 byte array aka `[u8; 32]`.

```rust
#[derive(Encode, Decode, Debug)]
struct Kitty {
    id: [u8; 32],
    price: u128,
}

// the part after the colon explicitly tells rust the expected type.
let kitty: Kitty = Decode::decode(&mut res_vec.as_slice()).unwrap();
println!("[+] Cute decoded Kitty: {:?}\n", kitty);
```

In order to use the `Decode::decode` function we need to add the `Parity`'s `Codec` crate as a dependency to the `Cargo.toml`. This is a basic Rust thing, and shall be left as an exercise to the reader.

This concludes this little tutorial. We went through important features of the `substrate-api-client`, namely creating extrinsics for custom runtime modules, reading storage values and decoding custom storage values. However, there are more features to our substrate api client such as predefined extrinsics for the `srml-contract` module, examples on how to listen to runtime `events` and more. The full code for this tutorial is found in the tutorials folder of the [substrate-api-client](https://github.com/scs/substrate-api-client).
