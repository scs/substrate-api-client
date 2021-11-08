use codec::{Decode, Encode};

pub mod config;
pub mod error;
pub mod events;
pub mod metadata;

/// Call trait.
pub trait Call: Encode {
    /// Pallet name.
    const PALLET: &'static str;
    /// Function name.
    const FUNCTION: &'static str;

    /// Returns true if the given pallet and function names match this call.
    fn is_call(pallet: &str, function: &str) -> bool {
        Self::PALLET == pallet && Self::FUNCTION == function
    }
}

/// Wraps an already encoded byte vector, prevents being encoded as a raw byte vector as part of
/// the transaction payload
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Encoded(pub Vec<u8>);

impl codec::Encode for Encoded {
    fn encode(&self) -> Vec<u8> {
        self.0.to_owned()
    }
}

/// A phase of a block's execution.
#[derive(Clone, Debug, Eq, PartialEq, Decode)]
pub enum Phase {
    /// Applying an extrinsic.
    ApplyExtrinsic(u32),
    /// Finalizing the block.
    Finalization,
    /// Initializing the block.
    Initialization,
}
