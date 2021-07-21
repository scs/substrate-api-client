/*
   Copyright 2019 Supercomputing Systems AG

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
use serde::{Deserialize, Serialize};

#[cfg(feature = "ws-client")]
pub use ws_client::{EventsError, WsRpcClient};

use crate::Balance;
#[cfg(feature = "ws-client")]
mod ws_client;

pub mod json_req;

#[derive(Debug, thiserror::Error)]
pub enum RpcClientError {
    #[error("Serde json error: {0}")]
    Serde(#[from] serde_json::error::Error),
    #[error("Extrinsic Error: {0}")]
    Extrinsic(String),
    #[error("mpsc send Error: {0}")]
    Send(#[from] std::sync::mpsc::SendError<String>),
}

#[derive(Debug, PartialEq)]
pub enum XtStatus {
    Finalized,
    InBlock,
    Broadcast,
    Ready,
    Future,
    Error,
    Unknown,
}

// Exact structure from
// https://github.com/paritytech/substrate/blob/master/client/rpc-api/src/state/helpers.rs
// Adding manually so we don't need sc-rpc-api, which brings in async dependencies
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadProof<Hash> {
    /// Block hash used to generate the proof
    pub at: Hash,
    /// A proof used to prove that storage entries are included in the storage trie
    pub proof: Vec<sp_core::Bytes>,
}

// Based on structures here
// https://github.com/paritytech/substrate/blob/cf086eeb894e61b45c732b3dad2ce4dcc54fa1af/frame/transaction-payment/src/types.rs#L68
// Would require pallet-transaction-payment and sp-rpc (for NumberOrHex) since the RPC returns either a number or hex string
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeDetails {
    /// The minimum fee for a transaction to be included in a block.
    pub inclusion_fee: Option<InclusionFee>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InclusionFee {
    /// This is the minimum amount a user pays for a transaction. It is declared
    /// as a base _weight_ in the runtime and converted to a fee using `WeightToFee`.
    #[serde(deserialize_with = "deserialize_number_or_hex")]
    pub base_fee: Balance,
    /// The length fee, the amount paid for the encoded length (in bytes) of the transaction.
    #[serde(deserialize_with = "deserialize_number_or_hex")]
    pub len_fee: Balance,
    /// - `targeted_fee_adjustment`: This is a multiplier that can tune the final fee based on
    ///     the congestion of the network.
    /// - `weight_fee`: This amount is computed based on the weight of the transaction. Weight
    /// accounts for the execution time of a transaction.
    ///
    /// adjusted_weight_fee = targeted_fee_adjustment * weight_fee
    #[serde(deserialize_with = "deserialize_number_or_hex")]
    pub adjusted_weight_fee: Balance,
}

impl FeeDetails {
    pub fn final_fee(&self) -> Balance {
        if let Some(inclusion_fee) = &self.inclusion_fee {
            inclusion_fee
                .base_fee
                .saturating_add(inclusion_fee.len_fee)
                .saturating_add(inclusion_fee.adjusted_weight_fee)
        } else {
            0
        }
    }
}

fn deserialize_number_or_hex<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: &str = serde::de::Deserialize::deserialize(deserializer)?;
    let num = if let Some(hex) = s.strip_prefix("0x") {
        u128::from_str_radix(hex, 16)
    } else {
        s.parse::<u128>()
    };
    num.map_err(|e| serde::de::Error::custom(e.to_string()))
}
