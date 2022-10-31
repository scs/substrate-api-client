use codec::{Decode, Encode};
use sp_runtime::AccountId32;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug)]
pub struct PayoutStakers {
    pub validator_stash: AccountId32,
    pub era: u32,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug)]
pub struct Batch {
    pub calls: Vec<([u8; 2], PayoutStakers)>,
}
