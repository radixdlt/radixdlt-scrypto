#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::types::RoyaltyAmount;
use sbor::rust::prelude::*;

use crate::*;

/// Royalty rules
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Default, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct ComponentRoyaltyConfig {
    pub royalty_amounts: BTreeMap<String, (RoyaltyAmount, bool)>,
}

/// Royalty rules
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum PackageRoyalty {
    Disabled,
    Enabled(BTreeMap<String, RoyaltyAmount>),
}

/// Royalty rules
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum PackageRoyaltyConfig {
    Disabled,
    Enabled(BTreeMap<String, RoyaltyAmount>),
}

impl Default for PackageRoyaltyConfig {
    fn default() -> Self {
        PackageRoyaltyConfig::Disabled
    }
}
