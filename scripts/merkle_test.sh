echo "testing merkle airdrop"

echo "Only use this on local dev chains - it doesn't really check the merkle root of the file, or the airdrop ID'"
echo "But it will still work when run on local"

echo "\n***Generate Merkle Tree***\n\n\n"

RUST_LOG=info cargo run -p ac-examples-async --example merkle_airdrop_cli -- generate-merkle-tree -i examples/async/examples/sample-claims.json -o tree-output_4.json

echo "\n***Create Airdrop***\n\n\n"

RUST_LOG=info cargo run -p ac-examples-async --example merkle_airdrop_cli -- create-airdrop -m 0x67d10c8ef788adf3c6760c0a751777fc042d6e91d90d10bcdedb814b077fce52

echo "\n***Fund Airdrop***\n\n\n"

RUST_LOG=info cargo run -p ac-examples-async --example merkle_airdrop_cli -- fund-airdrop -i 0 -a 6000000000000

echo "\n***Claim Airdrop***\n\n\n"

# claim #3
# RUST_LOG=info cargo run -p ac-examples-async --example merkle_airdrop_cli -- claim -i 0 -a 3000000000000 -p 0xb16783018bc11e3e97c5374886be31257ee97c96d94a870a94873f270eb8ef0a

# claim #1
RUST_LOG=info cargo run -p ac-examples-async --example merkle_airdrop_cli -- claim -i 0 -a 1000000000000 -p "0xbd1a69195ae32179a13667448671dec87c32e4b8cc507a6fa1886dd0f342f30d" -p "0xa42f58b57dd6ec7f8b2c1e165ba3dc1c452c98c262714866dfb634f8f21b42d3"