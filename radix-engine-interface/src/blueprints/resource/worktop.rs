use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::math::Decimal;
use crate::*;
use radix_engine_common::types::*;
use sbor::rust::prelude::*;

pub const WORKTOP_BLUEPRINT: &str = "Worktop";

pub const WORKTOP_DROP_IDENT: &str = "Worktop_drop";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct WorktopDropInput {
    pub worktop: Own,
}

pub type WorktopDropOutput = ();

pub const WORKTOP_PUT_IDENT: &str = "Worktop_put";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct WorktopPutInput {
    pub bucket: Bucket,
}

pub type WorktopPutOutput = ();

impl Clone for WorktopPutInput {
    fn clone(&self) -> Self {
        Self {
            bucket: Bucket(self.bucket.0),
        }
    }
}

pub const WORKTOP_TAKE_IDENT: &str = "Worktop_take";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct WorktopTakeInput {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

pub type WorktopTakeOutput = Bucket;

pub const WORKTOP_TAKE_NON_FUNGIBLES_IDENT: &str = "Worktop_take_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct WorktopTakeNonFungiblesInput {
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

pub type WorktopTakeNonFungiblesOutput = Bucket;

pub const WORKTOP_TAKE_ALL_IDENT: &str = "Worktop_take_all";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct WorktopTakeAllInput {
    pub resource_address: ResourceAddress,
}

pub type WorktopTakeAllOutput = Bucket;

pub const WORKTOP_ASSERT_CONTAINS_IDENT: &str = "Worktop_assert_contains";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct WorktopAssertContainsInput {
    pub resource_address: ResourceAddress,
}

pub type WorktopAssertContainsOutput = ();

pub const WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT: &str = "Worktop_assert_contains_amount";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct WorktopAssertContainsAmountInput {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

pub type WorktopAssertContainsAmountOutput = ();

pub const WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT: &str =
    "Worktop_assert_contains_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct WorktopAssertContainsNonFungiblesInput {
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

pub type WorktopAssertContainsNonFungiblesOutput = ();

pub const WORKTOP_DRAIN_IDENT: &str = "Worktop_drain";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct WorktopDrainInput {}

pub type WorktopDrainOutput = Vec<Bucket>;
