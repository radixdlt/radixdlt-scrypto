use crate::blueprints::resource::Bucket;
use crate::internal_prelude::*;
use crate::types::*;
use radix_common::data::scrypto::model::Own;
use sbor::rust::fmt::Debug;
use sbor::rust::prelude::*;

pub const COMPONENT_ROYALTY_SETTER_ROLE: &str = "royalty_setter";
pub const COMPONENT_ROYALTY_SETTER_UPDATER_ROLE: &str = "royalty_setter_updater";

pub const COMPONENT_ROYALTY_LOCKER_ROLE: &str = "royalty_locker";
pub const COMPONENT_ROYALTY_LOCKER_UPDATER_ROLE: &str = "royalty_locker_updater";

pub const COMPONENT_ROYALTY_CLAIMER_ROLE: &str = "royalty_claimer";
pub const COMPONENT_ROYALTY_CLAIMER_UPDATER_ROLE: &str = "royalty_claimer_updater";

pub const COMPONENT_ROYALTY_BLUEPRINT: &str = "ComponentRoyalty";

pub const COMPONENT_ROYALTY_CREATE_IDENT: &str = "create";

#[cfg_attr(
    feature = "fuzzing",
    derive(arbitrary::Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct ComponentRoyaltyCreateInput {
    pub royalty_config: ComponentRoyaltyConfig,
}

pub type ComponentRoyaltyCreateManifestInput = ComponentRoyaltyCreateInput;

pub type ComponentRoyaltyCreateOutput = Own;

pub const COMPONENT_ROYALTY_SET_ROYALTY_IDENT: &str = "set_royalty";

#[cfg_attr(
    feature = "fuzzing",
    derive(arbitrary::Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct ComponentRoyaltySetInput {
    pub method: String,
    pub amount: RoyaltyAmount,
}

pub type ComponentRoyaltySetManifestInput = ComponentRoyaltySetInput;

pub type ComponentRoyaltySetOutput = ();

pub const COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT: &str = "lock_royalty";

#[cfg_attr(
    feature = "fuzzing",
    derive(arbitrary::Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct ComponentRoyaltyLockInput {
    pub method: String,
}

pub type ComponentRoyaltyLockManifestInput = ComponentRoyaltyLockInput;

pub type ComponentRoyaltyLockOutput = ();

pub const COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT: &str = "claim_royalties";

#[cfg_attr(
    feature = "fuzzing",
    derive(arbitrary::Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct ComponentClaimRoyaltiesInput {}

pub type ComponentClaimRoyaltiesManifestInput = ComponentClaimRoyaltiesInput;

pub type ComponentClaimRoyaltiesOutput = Bucket;
