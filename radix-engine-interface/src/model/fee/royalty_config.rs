use sbor::rust::collections::HashMap;
use sbor::rust::str;
use sbor::rust::string::String;
use sbor::rust::string::ToString;

use crate::math::*;
use crate::scrypto;

/// Royalty rules
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub struct RoyaltyConfig {
    pub rules: HashMap<String, Decimal>,
    pub default_rule: Decimal,
}

impl Default for RoyaltyConfig {
    fn default() -> Self {
        Self {
            rules: HashMap::new(),
            default_rule: Decimal::zero(),
        }
    }
}

impl RoyaltyConfig {
    pub fn get_rule(&self, method_name: &str) -> &Decimal {
        self.rules.get(method_name).unwrap_or(&self.default_rule)
    }
}

pub struct RoyaltyConfigBuilder {
    rules: HashMap<String, Decimal>,
}

impl RoyaltyConfigBuilder {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(mut self, method: &str, amount: Decimal) -> Self {
        self.rules.insert(method.to_string(), amount);
        self
    }

    pub fn default(self, amount: Decimal) -> RoyaltyConfig {
        RoyaltyConfig {
            rules: self.rules,
            default_rule: amount,
        }
    }
}
