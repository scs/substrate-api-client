#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

pub use extrinsics::*;

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

/// Redefinition from `pallet-balances`. Currently, pallets break `no_std` builds, see:
/// https://github.com/paritytech/substrate/issues/8891
#[derive(Clone, Eq, PartialEq, Default, Debug, Encode, Decode)]
pub struct AccountDataGen<Balance> {
    /// Non-reserved part of the balance. There may still be restrictions on this, but it is the
    /// total pool what may in principle be transferred, reserved and used for tipping.
    ///
    /// This is the only balance that matters in terms of most operations on tokens. It
    /// alone is used to determine the balance when in the contract execution environment.
    pub free: Balance,
    /// Balance which is reserved and may not be used at all.
    ///
    /// This can still get slashed, but gets slashed last of all.
    ///
    /// This balance is a 'reserve' balance that other subsystems use in order to set aside tokens
    /// that are still 'owned' by the account holder, but which are suspendable.
    pub reserved: Balance,
    /// The amount that `free` may not drop below when withdrawing for *anything except transaction
    /// fee payment*.
    pub misc_frozen: Balance,
    /// The amount that `free` may not drop below when withdrawing specifically for transaction
    /// fee payment.
    pub fee_frozen: Balance,
}

/// Type used to encode the number of references an account has.
pub type RefCount = u32;

/// Redefinition from `frame-system`. Again see: https://github.com/paritytech/substrate/issues/8891
#[derive(Clone, Eq, PartialEq, Default, Debug, Encode, Decode)]
pub struct AccountInfoGen<Index, AccountData> {
    /// The number of transactions this account has sent.
    pub nonce: Index,
    /// The number of other modules that currently depend on this account's existence. The account
    /// cannot be reaped until this is zero.
    pub consumers: RefCount,
    /// The number of other modules that allow this account to exist. The account may not be reaped
    /// until this and `sufficients` are both zero.
    pub providers: RefCount,
    /// The number of modules that allow this account to exist for their own purposes only. The
    /// account may not be reaped until this and `providers` are both zero.
    pub sufficients: RefCount,
    /// The additional data that belongs to this account. Used to store the balance(s) in a lot of
    /// chains.
    pub data: AccountData,
}

pub type AccountData = AccountDataGen<Balance>;
pub type AccountInfo = AccountInfoGen<Index, AccountData>;
