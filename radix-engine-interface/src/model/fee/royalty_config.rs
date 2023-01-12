use sbor::rust::collections::HashMap;
use sbor::rust::str;
use sbor::rust::string::String;
use sbor::rust::string::ToString;

use crate::*;

/// Royalty rules
#[derive(
    Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe,
)]
pub struct RoyaltyConfig {
    pub rules: HashMap<String, u32>,
    pub default_rule: u32,
}

impl Default for RoyaltyConfig {
    fn default() -> Self {
        Self {
            rules: HashMap::new(),
            default_rule: 0,
        }
    }
}

impl RoyaltyConfig {
    pub fn get_rule(&self, method_name: &str) -> &u32 {
        self.rules.get(method_name).unwrap_or(&self.default_rule)
    }
}

pub struct RoyaltyConfigBuilder {
    rules: HashMap<String, u32>,
}

impl RoyaltyConfigBuilder {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(mut self, method: &str, amount: u32) -> Self {
        self.rules.insert(method.to_string(), amount);
        self
    }

    pub fn default(self, amount: u32) -> RoyaltyConfig {
        RoyaltyConfig {
            rules: self.rules,
            default_rule: amount,
        }
    }
}
