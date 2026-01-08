use radix_common::types::RoyaltyAmount;
use sbor::rust::prelude::*;

use crate::internal_prelude::*;

/// Royalty rules
#[cfg_attr(
    feature = "fuzzing",
    derive(::arbitrary::Arbitrary, ::serde::Serialize, ::serde::Deserialize)
)]
#[derive(Debug, Clone, Default, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct ComponentRoyaltyConfig {
    pub royalty_amounts: IndexMap<String, (RoyaltyAmount, bool)>,
}

/// Royalty rules
#[cfg_attr(
    feature = "fuzzing",
    derive(::arbitrary::Arbitrary, ::serde::Serialize, ::serde::Deserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum PackageRoyalty {
    Disabled,
    Enabled(IndexMap<String, RoyaltyAmount>),
}

/// Royalty rules
#[cfg_attr(
    feature = "fuzzing",
    derive(::arbitrary::Arbitrary, ::serde::Serialize, ::serde::Deserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor, Default)]
pub enum PackageRoyaltyConfig {
    #[default]
    Disabled,
    Enabled(IndexMap<String, RoyaltyAmount>),
}
