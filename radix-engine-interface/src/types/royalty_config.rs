#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use sbor::rust::collections::BTreeMap;
use sbor::rust::str;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::*;

use crate::*;

/// Royalty rules
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Sbor)]
pub struct RoyaltyConfig {
    pub rules: BTreeMap<String, u32>,
}

impl Default for RoyaltyConfig {
    fn default() -> Self {
        Self {
            rules: BTreeMap::new(),
        }
    }
}

impl RoyaltyConfig {
    pub fn get_rule(&self, method_name: &str) -> u32 {
        self.rules.get(method_name).cloned().unwrap_or(0)
    }

    pub fn set_rule<S: ToString>(&mut self, method: S, amount: u32) {
        self.rules.insert(method.to_string(), amount);
    }
}
