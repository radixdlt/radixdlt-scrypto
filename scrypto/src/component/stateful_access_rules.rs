use radix_engine_derive::scrypto;
use radix_engine_interface::api::types::ComponentId;
use radix_engine_interface::model::{AccessRules, AccessRule};

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub struct StatefulAccessRules {
    component: ComponentId,
    access_rules: AccessRules,
    index: usize,
}

impl StatefulAccessRules {
    pub fn new(component: ComponentId, access_rules: AccessRules, index: usize) -> Self {
        Self {
            component,
            access_rules,
            index,
        }
    }

    pub fn component(&self) -> ComponentId {
        self.component
    }

    pub fn access_rules(&self) -> AccessRules {
        self.access_rules.clone()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn set_method_auth(&mut self, _method_name: &str, _access_rule: AccessRule) {
        todo!();
    }

    pub fn set_default(&mut self, _access_rule: AccessRule) {
        todo!();
    }

    pub fn lock_method_auth(&mut self, _method_name: &str) {
        todo!();
    }

    pub fn lock_default(&mut self) {
        todo!();
    }
}
