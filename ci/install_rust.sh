#!/bin/bash

# Fail fast if any commands exists with error
# Print all executed commands
set -ex

# Download rustup script and execute it
curl https://sh.rustup.rs -sSf > ./rustup.sh
chmod +x ./rustup.sh
./rustup.sh -y

# Load new environment
source $HOME/.cargo/env

# Install nightly that supports clippy
# Overview: https://rust-lang.github.io/rustup-components-history/index.html
rustup default nightly-2020-02-06

# Install aux components, clippy for linter, rustfmt for formatting
rustup component add clippy
rustup component add rustfmt

# Install WASM toolchain
rustup target add wasm32-unknown-unknown

# Install wasm-gc
if ! [ -x "$(command -v wasm-gc)" ]; then
    cargo install --git https://github.com/alexcrichton/wasm-gc
else
    echo "wasm-gc already installed"
fi

# Show the installed versions
rustup show
