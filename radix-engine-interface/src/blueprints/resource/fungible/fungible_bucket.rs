use crate::math::*;
use crate::*;
use sbor::rust::fmt::Debug;

// Protected

pub const FUNGIBLE_BUCKET_BLUEPRINT: &str = "FungibleBucket";

pub const FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT: &str = "lock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleBucketLockAmountInput {
    pub amount: Decimal,
}

pub type FungibleBucketLockAmountOutput = ();

// Protected

pub const FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT: &str = "unlock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleBucketUnlockAmountInput {
    pub amount: Decimal,
}

pub type FungibleBucketUnlockAmountOutput = ();
