use crate::blueprints::resource::{Bucket, Proof};
use crate::data::scrypto::model::*;
use crate::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

pub const NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT: &str =
    "create_proof_of_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleBucketCreateProofOfNonFungiblesInput {
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub type NonFungibleBucketCreateProofOfNonFungiblesOutput = Proof;

pub const NON_FUNGIBLE_BUCKET_BLUEPRINT: &str = "NonFungibleBucket";

pub const NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_IDENT: &str = "take_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketTakeNonFungiblesInput {
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub type BucketTakeNonFungiblesOutput = Bucket;

pub const NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT: &str = "get_non_fungible_local_ids";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct BucketGetNonFungibleLocalIdsInput {}

pub type BucketGetNonFungibleLocalIdsOutput = BTreeSet<NonFungibleLocalId>;

// Protected

pub const NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT: &str = "lock_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleBucketLockNonFungiblesInput {
    pub local_ids: BTreeSet<NonFungibleLocalId>,
}

pub type NonFungibleBucketLockNonFungiblesOutput = ();

// Protected

pub const NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_IDENT: &str = "unlock_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleBucketUnlockNonFungiblesInput {
    pub local_ids: BTreeSet<NonFungibleLocalId>,
}

pub type NonFungibleBucketUnlockNonFungiblesOutput = ();
