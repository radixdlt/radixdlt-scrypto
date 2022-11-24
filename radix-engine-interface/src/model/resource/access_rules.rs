use radix_engine_interface::api::types::NativeFn;
use sbor::rust::collections::hash_map::Iter;
use sbor::rust::collections::HashMap;
use sbor::rust::str;
use sbor::rust::string::String;
use sbor::rust::string::ToString;

use crate::model::*;
use crate::scrypto;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub enum AccessRuleKey {
    ScryptoMethod(String),
    Native(NativeFn),
}

/// Method authorization rules for a component
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub struct AccessRules {
    method_auth: HashMap<AccessRuleKey, AccessRule>,
    default_auth: AccessRule,
}

impl AccessRules {
    pub fn new() -> Self {
        Self {
            method_auth: HashMap::new(),
            default_auth: AccessRule::DenyAll,
        }
    }

    pub fn get(&self, key: &AccessRuleKey) -> &AccessRule {
        self.method_auth.get(key).unwrap_or(&self.default_auth)
    }

    pub fn get_default(&self) -> &AccessRule {
        &self.default_auth
    }

    // TODO: Move into scrypto repo
    pub fn method(mut self, method_name: &str, method_auth: AccessRule) -> Self {
        self.method_auth.insert(
            AccessRuleKey::ScryptoMethod(method_name.to_string()),
            method_auth,
        );
        self
    }

    pub fn set_access_rule(&mut self, key: AccessRuleKey, method_auth: AccessRule) {
        self.method_auth.insert(key, method_auth);
    }

    pub fn default(mut self, method_auth: AccessRule) -> Self {
        self.default_auth = method_auth;
        self
    }

    pub fn iter(&self) -> Iter<'_, AccessRuleKey, AccessRule> {
        let l = self.method_auth.iter();
        l
    }
}
