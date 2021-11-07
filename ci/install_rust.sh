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

# Show the installed versions
rustup show
