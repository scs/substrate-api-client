#!/bin/bash
set -euo pipefail

echo "[+] Testing Application Crypto"
cargo build --release -p test-no-std --features application-crypto
echo "[+] Testing arithmetic"
cargo build --release -p test-no-std --features arithmetic
echo "[+] Testing beefy"
cargo build --release -p test-no-std --features beefy
echo "[+] Testing babe"
cargo build --release -p test-no-std --features babe
echo "[+] Testing slots"
cargo build --release -p test-no-std --features slots
echo "[+] Testing core"
cargo build --release -p test-no-std --features core
echo "[+] Testing finality-grandpa"
cargo build --release -p test-no-std --features finality-grandpa
echo "[+] Testing mmr"
cargo build --release -p test-no-std --features mmr
echo "[+] Testing npos-elections"
cargo build --release -p test-no-std --features npos-elections
echo "[+] Testing rpc"
cargo build --release -p test-no-std --features rpc
echo "[+] Testing runtime"
cargo build --release -p test-no-std --features runtime
echo "[+] Testing serializer"
cargo build --release -p test-no-std --features serializer
echo "[+] Testing test-primitives"
cargo build --release -p test-no-std --features test-primitives
echo "[+] Testing version"
cargo build --release -p test-no-std --features version
echo "[+] Testing weights"
cargo build --release -p test-no-std --features weights
echo "[+] Testing keystore"
cargo build --release -p test-no-std --features keystore
