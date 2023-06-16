#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::types::RoyaltyAmount;
use sbor::rust::prelude::*;

use crate::*;

/// Royalty rules
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct RoyaltyConfig {
    #[cfg(feature = "indexmap")]
    pub rules: sbor::prelude::IndexMap<String, RoyaltyAmount>,
    #[cfg(not(feature = "indexmap"))]
    pub rules: BTreeMap<String, RoyaltyAmount>,
}

impl Default for RoyaltyConfig {
    #[cfg(feature = "indexmap")]
    fn default() -> Self {
        Self {
            rules: sbor::prelude::index_map::new(),
        }
    }

    #[cfg(not(feature = "indexmap"))]
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
