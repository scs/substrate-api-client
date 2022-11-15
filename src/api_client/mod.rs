use crate::rpc::json_req;
pub use crate::{
	api_client::{
		error::{ApiResult, Error as ApiClientError},
		rpc::XtStatus,
	},
	utils::FromHexString,
};
use ac_node_api::metadata::{Metadata, MetadataError};
use codec::{Decode, Encode};
use log::{debug, info};
pub use metadata::RuntimeMetadataPrefixed;
use serde::de::DeserializeOwned;
pub use serde_json::Value;
use sp_core::H256 as Hash;
pub use sp_core::{crypto::Pair, storage::StorageKey};
use sp_rpc::number::NumberOrHex;
pub use sp_runtime::{
	generic::SignedBlock,
	traits::{Block, Header, IdentifyAccount},
	AccountId32 as AccountId, MultiSignature, MultiSigner,
};
pub use sp_std::prelude::*;
pub use sp_version::RuntimeVersion;
pub use transaction_payment::FeeDetails;
use transaction_payment::{InclusionFee, RuntimeDispatchInfo};

pub mod api;
pub mod api_traits;
pub mod error;
pub mod rpc;

pub type AccountInfoFor<T> = system::AccountInfo<
	<Runtime as system::Config>::Index,
	<Runtime as system::Config>::AccountData,
>;

// Simplified structure from
// https://github.com/paritytech/substrate/blob/master/client/rpc-api/src/state/helpers.rs
// Adding manually so we don't need sc-rpc-api, which brings in async dependencies
#[derive(Debug, PartialEq, Eq)]
pub struct ReadProof<Hash> {
	/// Block hash used to generate the proof
	pub at: Hash,
	/// A proof used to prove that storage entries are included in the storage trie
	pub proof: Vec<sp_core::Bytes>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum XtStatus {
	Unknown = 0,
	/// uses `author_submit` without watching.
	SubmitOnly = 1,
	Ready = 2,
	Broadcast = 3,
	InBlock = 4,
	Finalized = 5,
	Future = 10,
}
