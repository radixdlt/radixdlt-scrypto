use crate::blueprints::resource::{Bucket, Proof};
use crate::internal_prelude::*;
use radix_common::data::scrypto::model::*;
use sbor::rust::collections::IndexSet;
use sbor::rust::fmt::Debug;

pub const NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT: &str =
    "create_proof_of_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct NonFungibleBucketCreateProofOfNonFungiblesInput {
    pub ids: IndexSet<NonFungibleLocalId>,
}

pub type NonFungibleBucketCreateProofOfNonFungiblesManifestInput =
    NonFungibleBucketCreateProofOfNonFungiblesInput;

pub type NonFungibleBucketCreateProofOfNonFungiblesOutput = Proof;

pub const NON_FUNGIBLE_BUCKET_BLUEPRINT: &str = "NonFungibleBucket";

pub const NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_IDENT: &str = "take_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct BucketTakeNonFungiblesInput {
    pub ids: IndexSet<NonFungibleLocalId>,
}

pub type BucketTakeNonFungiblesManifestInput = BucketTakeNonFungiblesInput;

pub type BucketTakeNonFungiblesOutput = Bucket;

pub const NON_FUNGIBLE_BUCKET_CONTAINS_NON_FUNGIBLE_IDENT: &str = "contains_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct NonFungibleBucketContainsNonFungibleInput {
    pub id: NonFungibleLocalId,
}

pub type NonFungibleBucketContainsNonFungibleManifestInput =
    NonFungibleBucketContainsNonFungibleInput;

pub type NonFungibleBucketContainsNonFungibleOutput = bool;

pub const NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT: &str = "get_non_fungible_local_ids";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct BucketGetNonFungibleLocalIdsInput {}

pub type BucketGetNonFungibleLocalIdsManifestInput = BucketGetNonFungibleLocalIdsInput;

pub type BucketGetNonFungibleLocalIdsOutput = IndexSet<NonFungibleLocalId>;

pub const NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT: &str = "lock_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct NonFungibleBucketLockNonFungiblesInput {
    pub local_ids: IndexSet<NonFungibleLocalId>,
}

pub type NonFungibleBucketLockNonFungiblesManifestInput = NonFungibleBucketLockNonFungiblesInput;

pub type NonFungibleBucketLockNonFungiblesOutput = ();

pub const NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_IDENT: &str = "unlock_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct NonFungibleBucketUnlockNonFungiblesInput {
    pub local_ids: IndexSet<NonFungibleLocalId>,
}

pub type NonFungibleBucketUnlockNonFungiblesManifestInput =
    NonFungibleBucketUnlockNonFungiblesInput;

pub type NonFungibleBucketUnlockNonFungiblesOutput = ();

pub type NonFungibleBucketPutInput = BucketPutInput;
pub type NonFungibleBucketPutManifestInput = BucketPutManifestInput;

pub type NonFungibleBucketGetAmountInput = BucketGetAmountInput;
pub type NonFungibleBucketGetAmountManifestInput = BucketGetAmountManifestInput;

pub type NonFungibleBucketGetResourceAddressInput = BucketGetResourceAddressInput;
pub type NonFungibleBucketGetResourceAddressManifestInput = BucketGetResourceAddressManifestInput;

pub type NonFungibleBucketCreateProofOfAllInput = BucketCreateProofOfAllInput;
pub type NonFungibleBucketCreateProofOfAllManifestInput = BucketCreateProofOfAllManifestInput;

pub type NonFungibleBucketGetNonFungibleLocalIdsInput = BucketGetNonFungibleLocalIdsInput;
pub type NonFungibleBucketGetNonFungibleLocalIdsManifestInput =
    BucketGetNonFungibleLocalIdsManifestInput;
