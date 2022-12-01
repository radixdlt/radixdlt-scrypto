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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub enum AccessRuleEntry {
    AccessRule(AccessRule),
    Group(String),
}

/// Method authorization rules for a component
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub struct AccessRules {
    method_auth: HashMap<AccessRuleKey, AccessRuleEntry>,
    grouped_auth: HashMap<String, AccessRule>,
    default_auth: AccessRule,
    method_auth_mutability: HashMap<AccessRuleKey, AccessRule>,
    grouped_auth_mutability: HashMap<String, AccessRule>,
    default_auth_mutability: AccessRule,
}

impl AccessRules {
    pub fn new() -> Self {
        Self {
            method_auth: HashMap::new(),
            grouped_auth: HashMap::new(),
            default_auth: AccessRule::DenyAll,
            method_auth_mutability: HashMap::new(),
            grouped_auth_mutability: HashMap::new(),
            default_auth_mutability: AccessRule::DenyAll,
        }
    }

    // TODO: Move into scrypto repo as a builder
    pub fn method(mut self, method_name: &str, method_auth: AccessRule) -> Self {
        self.method_auth.insert(
            AccessRuleKey::ScryptoMethod(method_name.to_string()),
            AccessRuleEntry::AccessRule(method_auth),
        );
        self
    }

    // TODO: Move into scrypto repo as a builder
    pub fn default(
        mut self,
        default_auth: AccessRule,
        default_auth_mutability: AccessRule,
    ) -> Self {
        self.default_auth = default_auth;
        self.default_auth_mutability = default_auth_mutability;
        self
    }

    pub fn set_default_auth(&mut self, default_auth: AccessRule) {
        self.default_auth = default_auth;
    }

    pub fn set_default_auth_mutability(&mut self, default_auth_mutability: AccessRule) {
        self.default_auth_mutability = default_auth_mutability;
    }

    pub fn get_mutability(&self, key: &AccessRuleKey) -> &AccessRule {
        self.method_auth_mutability
            .get(key)
            .unwrap_or(&self.default_auth_mutability)
    }

    pub fn get_group_mutability(&self, key: &str) -> &AccessRule {
        self.grouped_auth_mutability
            .get(key)
            .unwrap_or(&self.default_auth_mutability)
    }

    pub fn set_mutability(&mut self, key: AccessRuleKey, method_auth: AccessRule) {
        self.method_auth_mutability.insert(key, method_auth);
    }

    pub fn set_group_mutability(&mut self, key: String, method_auth: AccessRule) {
        self.grouped_auth_mutability.insert(key, method_auth);
    }

    pub fn get(&self, key: &AccessRuleKey) -> &AccessRule {
        match self.method_auth.get(key) {
            None => &self.default_auth,
            Some(AccessRuleEntry::AccessRule(access_rule)) => access_rule,
            Some(AccessRuleEntry::Group(group_key)) => self.get_group(group_key),
        }
    }

    pub fn get_group(&self, key: &str) -> &AccessRule {
        self.grouped_auth.get(key).unwrap_or(&self.default_auth)
    }

    pub fn get_default(&self) -> &AccessRule {
        &self.default_auth
    }

    pub fn set_method_access_rule(&mut self, key: AccessRuleKey, access_rule: AccessRule) {
        self.method_auth
            .insert(key, AccessRuleEntry::AccessRule(access_rule));
    }

    pub fn set_group_access_rule(&mut self, group_key: String, access_rule: AccessRule) {
        self.grouped_auth.insert(group_key, access_rule);
    }

    pub fn set_group_access_rule_and_mutability(
        &mut self,
        group_key: String,
        access_rule: AccessRule,
        mutability: AccessRule,
    ) {
        self.grouped_auth.insert(group_key.clone(), access_rule);
        self.grouped_auth_mutability.insert(group_key, mutability);
    }

    pub fn set_access_rule_and_mutability(
        &mut self,
        key: AccessRuleKey,
        access_rule: AccessRule,
        mutability: AccessRule,
    ) {
        self.method_auth
            .insert(key.clone(), AccessRuleEntry::AccessRule(access_rule));
        self.method_auth_mutability.insert(key, mutability);
    }

    pub fn set_group_and_mutability(
        &mut self,
        key: AccessRuleKey,
        group: String,
        mutability: AccessRule,
    ) {
        self.method_auth
            .insert(key.clone(), AccessRuleEntry::Group(group));
        self.method_auth_mutability.insert(key, mutability);
    }

    pub fn iter(&self) -> Iter<'_, AccessRuleKey, AccessRuleEntry> {
        let l = self.method_auth.iter();
        l
    }
}
