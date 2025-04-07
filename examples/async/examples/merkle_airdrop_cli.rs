/*
    Copyright 2024 Resonance Network
    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

//! CLI tool for testing the merkle-airdrop pallet in the Resonance Network.
//! Provides functionality to:
//! 1. Generate Merkle trees, roots, and proofs from a list of claims
//! 2. Create new airdrops with a Merkle root
//! 3. Fund existing airdrops with a specified amount
//! 4. Claim rewards from airdrops using Merkle proofs

use clap::{Parser, Subcommand};
use substrate_api_client::{
    Api, XtStatus,
    rpc::JsonrpseeClient,
    ac_primitives::ResonanceRuntimeConfig,
    SubmitAndWatch,
    ac_compose_macros::compose_extrinsic,
};
use dilithium_crypto::pair::{crystal_alice};
use sp_core::H256;
use std::{fs, fmt};
use log::info;
use blake2::{Blake2b512, Digest};

#[derive(Debug)]
enum CliError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Hex(hex::FromHexError),
    Api(substrate_api_client::Error),
    Rpc(substrate_api_client::rpc::Error),
    Custom(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(e) => write!(f, "IO error: {}", e),
            CliError::Json(e) => write!(f, "JSON error: {}", e),
            CliError::Hex(e) => write!(f, "Hex decoding error: {}", e),
            CliError::Api(e) => write!(f, "API error: {:?}", e),
            CliError::Rpc(e) => write!(f, "RPC error: {:?}", e),
            CliError::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::Io(e)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError::Json(e)
    }
}

impl From<hex::FromHexError> for CliError {
    fn from(e: hex::FromHexError) -> Self {
        CliError::Hex(e)
    }
}

impl From<substrate_api_client::Error> for CliError {
    fn from(e: substrate_api_client::Error) -> Self {
        CliError::Api(e)
    }
}

impl From<substrate_api_client::rpc::Error> for CliError {
    fn from(e: substrate_api_client::rpc::Error) -> Self {
        CliError::Rpc(e)
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,

    #[arg(short, long, default_value = "ws://127.0.0.1:9944")]
    node_url: String,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create a merkle tree and output root and proofs
    GenerateMerkleTree {
        #[arg(short, long)]
        input_file: String,
        
        #[arg(short, long)]
        output_file: Option<String>,
    },
    
    /// Create a new airdrop with the given merkle root
    CreateAirdrop {
        #[arg(short, long)]
        merkle_root: String,
    },
    
    /// Fund an existing airdrop
    FundAirdrop {
        #[arg(short, long)]
        id: u32,
        
        #[arg(short, long)]
        amount: u128,
    },
    
    /// Claim from an airdrop
    Claim {
        #[arg(short, long)]
        id: u32,
        
        #[arg(short, long)]
        amount: u128,
        
        #[arg(short, long)]
        proofs: Vec<String>,
    },
}

#[derive(serde::Deserialize, Debug)]
struct ClaimInput {
    address: String,
    amount: String,
}

#[derive(serde::Serialize, Debug)]
struct MerkleOutput {
    root: String,
    claims: Vec<ClaimData>,
}

#[derive(serde::Serialize, Debug)]
struct ClaimData {
    address: String,
    amount: String,
    proof: Vec<String>,
}

/// Simple hash function for leaf nodes
fn hash_leaf(address: &str, amount: &str) -> H256 {
    let mut hasher = Blake2b512::new();
    hasher.update(address.as_bytes());
    hasher.update(amount.as_bytes());
    let result = hasher.finalize();
    
    // Take first 32 bytes for H256
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result[0..32]);
    H256::from(bytes)
}

/// Hash internal nodes by concatenating and hashing them
fn hash_nodes(left: &H256, right: &H256) -> H256 {
    let mut hasher = Blake2b512::new();
    hasher.update(left.as_bytes());
    hasher.update(right.as_bytes());
    let result = hasher.finalize();
    
    // Take first 32 bytes for H256
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result[0..32]);
    H256::from(bytes)
}

/// Build a Merkle tree and generate proofs
fn build_merkle_tree(claims: &[ClaimInput]) -> MerkleOutput {
    // Create leaf nodes
    let leaves: Vec<(H256, usize)> = claims
        .iter()
        .enumerate()
        .map(|(i, claim)| (hash_leaf(&claim.address, &claim.amount), i))
        .collect();
    
    // Save all intermediate nodes for proof generation
    let mut tree_nodes: Vec<Vec<H256>> = vec![leaves.iter().map(|(hash, _)| *hash).collect()];
    
    // Build the tree bottom-up
    while tree_nodes.last().unwrap().len() > 1 {
        let level = tree_nodes.last().unwrap();
        let mut next_level = Vec::new();
        
        for i in (0..level.len()).step_by(2) {
            if i + 1 < level.len() {
                // Hash pair of nodes
                let hash = hash_nodes(&level[i], &level[i + 1]);
                next_level.push(hash);
            } else {
                // Odd node, promote to next level
                next_level.push(level[i]);
            }
        }
        
        tree_nodes.push(next_level);
    }
    
    // Root is the last node in the last level
    let root = tree_nodes.last().unwrap()[0];
    
    // Generate proofs for each leaf
    let mut output_claims = Vec::new();
    for (i, claim) in claims.iter().enumerate() {
        let mut proof = Vec::new();
        let mut index = leaves[i].1;
        
        for level in 0..tree_nodes.len() - 1 {
            let is_right = index % 2 == 1;
            let sibling_idx = if is_right { index - 1 } else { index + 1 };
            
            if sibling_idx < tree_nodes[level].len() {
                proof.push(format!("0x{}", hex::encode(tree_nodes[level][sibling_idx].as_bytes())));
            }
            
            index /= 2;
        }
        
        output_claims.push(ClaimData {
            address: claim.address.clone(),
            amount: claim.amount.clone(),
            proof,
        });
    }
    
    MerkleOutput {
        root: format!("0x{}", hex::encode(root.as_bytes())),
        claims: output_claims,
    }
}

#[tokio::main]
async fn main() -> Result<(), CliError> {
    env_logger::init();
    let args = Args::parse();
    
    match args.command {
        Command::GenerateMerkleTree { input_file, output_file } => {
            info!("Generating Merkle tree from {}", input_file);
            
            // Read and parse input file
            let input_data = fs::read_to_string(input_file)?;
            let claims: Vec<ClaimInput> = serde_json::from_str(&input_data)?;
            
            info!("Processing {} claims", claims.len());
            let merkle_output = build_merkle_tree(&claims);
            
            // Output the Merkle tree data
            let output_json = serde_json::to_string_pretty(&merkle_output)?;
            
            match output_file {
                Some(file) => {
                    fs::write(&file, output_json)?;
                    info!("Merkle tree data written to {}", file);
                }
                None => {
                    println!("{}", output_json);
                }
            }
            
            Ok(())
        },
        
        Command::CreateAirdrop { merkle_root } => {
            info!("Connecting to node at {}", args.node_url);
            let client = JsonrpseeClient::new(&args.node_url).await?;
            let mut api = Api::<ResonanceRuntimeConfig, _>::new(client).await?;
            
            // Remove 0x prefix if present
            let merkle_root = merkle_root.trim_start_matches("0x");
            let merkle_bytes = hex::decode(merkle_root)?;
            
            // Ensure we have exactly 32 bytes
            if merkle_bytes.len() != 32 {
                return Err(CliError::Custom(format!("Merkle root must be exactly 32 bytes, got {}", merkle_bytes.len())));
            }
            
            let mut root = [0u8; 32];
            root.copy_from_slice(&merkle_bytes);
            let root = H256::from(root);
            
            info!("Creating airdrop with merkle root: {}", hex::encode(root.as_bytes()));
            
            // Set the signer
            let signer = crystal_alice();
            api.set_signer(signer.clone().into());
            info!("Using signer: {:?}", signer.public());
            
            // Store the cloned API
            let api_ref = api.clone();
            // Create and sign the extrinsic
            let xt = compose_extrinsic!(
                api_ref,
                "MerkleAirdrop",
                "create_airdrop",
                root
            ).ok_or_else(|| CliError::Custom("Failed to create extrinsic".to_string()))?;
            
            // Submit and wait for inclusion
            info!("Submitting createAirdrop transaction...");
            let result = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).await?;
            info!("Transaction included in block: {:?}", result);
            
            Ok(())
        },
        
        Command::FundAirdrop { id, amount } => {
            info!("Connecting to node at {}", args.node_url);
            let client = JsonrpseeClient::new(&args.node_url).await?;
            let mut api = Api::<ResonanceRuntimeConfig, _>::new(client).await?;
            
            info!("Funding airdrop {} with amount {}", id, amount);
            
            // Set the signer
            let signer = crystal_alice();
            api.set_signer(signer.clone().into());
            info!("Using signer: {:?}", signer.public());
            
            // Store the cloned API
            let api_ref = api.clone();
            // Create and sign the extrinsic
            let xt = compose_extrinsic!(
                api_ref,
                "MerkleAirdrop",
                "fund_airdrop",
                id,
                amount
            ).ok_or_else(|| CliError::Custom("Failed to create extrinsic".to_string()))?;
            
            // Submit and wait for inclusion
            info!("Submitting fundAirdrop transaction...");
            let result = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).await?;
            info!("Transaction included in block: {:?}", result);
            
            Ok(())
        },
        
        Command::Claim { id, amount, proofs } => {
            info!("Connecting to node at {}", args.node_url);
            let client = JsonrpseeClient::new(&args.node_url).await?;
            let mut api = Api::<ResonanceRuntimeConfig, _>::new(client).await?;
            
            info!("Claiming from airdrop {} for amount {}", id, amount);
            
            // Convert proof strings to bytes
            let proof_bytes: Vec<H256> = proofs
                .iter()
                .map(|p| {
                    let p = p.trim_start_matches("0x");
                    let bytes = hex::decode(p).expect("Invalid hex in proof");
                    let mut array = [0u8; 32];
                    array.copy_from_slice(&bytes);
                    H256::from(array)
                })
                .collect();
            
            // Set the signer
            let signer = crystal_alice();
            api.set_signer(signer.clone().into());
            info!("Using signer: {:?}", signer.public());
            
            // Store the cloned API
            let api_ref = api.clone();
            // Create and sign the extrinsic
            let xt = compose_extrinsic!(
                api_ref,
                "MerkleAirdrop",
                "claim",
                id,
                amount,
                proof_bytes
            ).ok_or_else(|| CliError::Custom("Failed to create extrinsic".to_string()))?;
            
            // Submit and wait for inclusion
            info!("Submitting claim transaction...");
            let result = api.submit_and_watch_extrinsic_until(xt, XtStatus::InBlock).await?;
            info!("Transaction included in block: {:?}", result);
            
            Ok(())
        },
    }
} 