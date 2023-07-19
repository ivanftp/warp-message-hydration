# CosmWasm Message Hydration

This repository contains a CosmWasm smart contract with a hydration function. The hydration function is used to replace template placeholders with actual values for processing.

## Quick Start

To get started with this contract, clone the repository and navigate to its directory:

```sh
git clone https://github.com/ivanftp/warp-message-hydration.git
cd <repo_directory>
```

## Prerequisites

Before starting, make sure you have [rustup](https://rustup.rs/) along with a
recent `rustc` and `cargo` version installed. Currently, we are testing on 1.58.1+.

And you need to have the `wasm32-unknown-unknown` target installed as well.

You can check that via:

```sh
rustc --version
cargo --version
rustup target list --installed
# if wasm32 is not listed above, run this
rustup target add wasm32-unknown-unknown
```

## Compiling the Contract

To compile the contract, use the following command:

```sh
cargo build
```

## Testing the Contract

You can run unit tests by executing:

```sh
cargo test
```
