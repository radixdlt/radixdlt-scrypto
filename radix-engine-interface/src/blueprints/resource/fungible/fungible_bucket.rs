use crate::blueprints::resource::Proof;
use crate::internal_prelude::*;
use radix_common::math::*;
use sbor::rust::prelude::*;

pub const FUNGIBLE_BUCKET_BLUEPRINT: &str = "FungibleBucket";

pub const FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT: &str = "lock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleBucketLockAmountInput {
    pub amount: Decimal,
}

pub type FungibleBucketLockAmountManifestInput = FungibleBucketLockAmountInput;

pub type FungibleBucketLockAmountOutput = ();

pub const FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT: &str = "unlock_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleBucketUnlockAmountInput {
    pub amount: Decimal,
}

pub type FungibleBucketUnlockAmountManifestInput = FungibleBucketUnlockAmountInput;

pub type FungibleBucketUnlockAmountOutput = ();

pub const FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT: &str = "create_proof_of_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleBucketCreateProofOfAmountInput {
    pub amount: Decimal,
}

pub type FungibleBucketCreateProofOfAmountManifestInput = FungibleBucketCreateProofOfAmountInput;

pub type FungibleBucketCreateProofOfAmountOutput = Proof;

pub type FungibleBucketPutInput = BucketPutInput;
pub type FungibleBucketPutManifestInput = BucketPutManifestInput;

pub type FungibleBucketGetAmountInput = BucketGetAmountInput;
pub type FungibleBucketGetAmountManifestInput = BucketGetAmountManifestInput;

pub type FungibleBucketGetResourceAddressInput = BucketGetResourceAddressInput;
pub type FungibleBucketGetResourceAddressManifestInput = BucketGetResourceAddressManifestInput;

pub type FungibleBucketCreateProofOfAllInput = BucketCreateProofOfAllInput;
pub type FungibleBucketCreateProofOfAllManifestInput = BucketCreateProofOfAllManifestInput;
