use crate::blueprints::resource::Proof;
use crate::math::*;
use crate::*;
use sbor::rust::prelude::*;

pub const FUNGIBLE_BUCKET_BLUEPRINT: &str = "FungibleBucket";

pub const FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT: &str = "lock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleBucketLockAmountInput {
    pub amount: Decimal,
}

pub type FungibleBucketLockAmountOutput = ();

pub const FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT: &str = "unlock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleBucketUnlockAmountInput {
    pub amount: Decimal,
}

pub type FungibleBucketUnlockAmountOutput = ();

pub const FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT: &str = "create_proof_of_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleBucketCreateProofOfAmountInput {
    pub amount: Decimal,
}

pub type FungibleBucketCreateProofOfAmountOutput = Proof;
