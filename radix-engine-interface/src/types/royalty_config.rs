#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::types::RoyaltyAmount;
use sbor::rust::prelude::*;

use crate::*;

/// Royalty rules
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct RoyaltyConfig {
    pub rules: BTreeMap<String, RoyaltyAmount>,
}

impl Default for RoyaltyConfig {
    fn default() -> Self {
        Self {
            rules: BTreeMap::new(),
        }
    }
}

impl RoyaltyConfig {
    pub fn get_rule(&self, method_name: &str) -> RoyaltyAmount {
        self.rules
            .get(method_name)
            .cloned()
            .unwrap_or(RoyaltyAmount::Free)
    }

    pub fn set_rule<S: ToString>(&mut self, method: S, amount: RoyaltyAmount) {
        self.rules.insert(method.to_string(), amount);
    }
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
