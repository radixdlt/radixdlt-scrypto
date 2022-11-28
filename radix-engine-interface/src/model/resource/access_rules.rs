use sbor::rust::collections::hash_map::Iter;
use sbor::rust::collections::HashMap;
use sbor::rust::str;
use sbor::rust::string::String;
use sbor::rust::string::ToString;

use crate::model::*;
use crate::scrypto;

/// Method authorization rules for a component
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub struct AccessRules {
    method_auth: HashMap<String, (AccessRule, Mutability)>,
    default_auth: (AccessRule, Mutability),
}

impl AccessRules {
    pub fn new() -> Self {
        Self {
            method_auth: HashMap::new(),
            default_auth: (AccessRule::DenyAll, LOCKED),
        }
    }

    pub fn get(&self, method_name: &str) -> &(AccessRule, Mutability) {
        self.method_auth
            .get(method_name)
            .unwrap_or(&self.default_auth)
    }

    pub fn get_default(&self) -> &(AccessRule, Mutability) {
        &self.default_auth
    }

    pub fn method(
        mut self,
        method_name: &str,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> Self {
        self.method_auth
            .insert(method_name.to_string(), (method_auth, mutability));
        self
    }

    pub fn default(mut self, method_auth: AccessRule, mutability: Mutability) -> Self {
        self.default_auth = (method_auth, mutability);
        self
    }

    pub fn iter(&self) -> Iter<'_, String, (AccessRule, Mutability)> {
        let l = self.method_auth.iter();
        l
    }
}
