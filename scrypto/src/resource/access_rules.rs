use sbor::rust::collections::hash_map::Iter;
use sbor::rust::collections::HashMap;
use sbor::rust::str;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::*;

use crate::resource::*;

/// Method authorization rules for a component
#[derive(Debug, Clone, PartialEq, Describe, TypeId, Encode, Decode)]
pub struct AccessRules {
    method_auth: HashMap<String, AccessRule>,
    default_auth: AccessRule,
}

impl AccessRules {
    pub fn new() -> Self {
        Self {
            method_auth: HashMap::new(),
            default_auth: AccessRule::DenyAll,
        }
    }

    pub fn get(&self, method_name: &str) -> &AccessRule {
        self.method_auth
            .get(method_name)
            .unwrap_or(&self.default_auth)
    }

    pub fn get_default(&self) -> &AccessRule {
        &self.default_auth
    }

    pub fn method(mut self, method_name: &str, method_auth: AccessRule) -> Self {
        self.method_auth
            .insert(method_name.to_string(), method_auth);
        self
    }

    pub fn default(mut self, method_auth: AccessRule) -> Self {
        self.default_auth = method_auth;
        self
    }

    pub fn iter(&self) -> Iter<'_, String, AccessRule> {
        let l = self.method_auth.iter();
        l
    }

    pub fn contains_dynamic_rules(&self) -> bool {
        self.get_default().contains_dynamic_rules()
            || self
                .iter()
                .map(|(_, access_rule)| access_rule)
                .any(|access_rule| access_rule.contains_dynamic_rules())
    }
}
