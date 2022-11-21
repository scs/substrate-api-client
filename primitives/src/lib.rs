#![cfg_attr(not(feature = "std"), no_std)]

pub use extrinsic_params::*;
pub use extrinsics::*;

pub mod extrinsic_params;
pub mod extrinsics;

/// The block number type used in this runtime.
pub type BlockNumber = u64;
/// The timestamp moment type used in this runtime.
pub type Moment = u64;
/// Index of a transaction.
//fixme: make generic
pub type Index = u32;

pub type Hash = sp_core::H256;

//fixme: make generic
pub type Balance = u128;

pub use frame_system::AccountInfo as GenericAccountInfo;
pub use pallet_balances::AccountData as GenericAccountData;

pub type AccountData = GenericAccountData<Balance>;
pub type AccountInfo = GenericAccountInfo<Index, AccountData>;
