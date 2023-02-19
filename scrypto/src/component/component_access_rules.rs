use crate::engine::scrypto_env::ScryptoEnv;
use radix_engine_derive::LegacyDescribe;
use radix_engine_derive::*;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesSetMethodAccessRuleInput, AccessRulesSetMethodMutabilityInput,
    ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT, ACCESS_RULES_SET_METHOD_MUTABILITY_IDENT,
};
use radix_engine_interface::api::types::{
    ComponentAddress, ComponentId, GlobalAddress, NodeModuleId, RENodeId, ToString,
};
use radix_engine_interface::api::ClientComponentApi;
use radix_engine_interface::blueprints::resource::{AccessRule, AccessRuleEntry, AccessRuleKey};
use radix_engine_interface::data::scrypto_encode;

// TODO: Should `Encode` and `Decode` be removed so that `ComponentAccessRules` can not be passed
// between components?
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, LegacyDescribe)]
pub struct ComponentAccessRules {
    component: ComponentIdentifier,
    index: u32,
}

impl ComponentAccessRules {
    pub(crate) fn new<T: Into<ComponentIdentifier>>(component: T, index: u32) -> Self {
        Self {
            component: component.into(),
            index,
        }
    }

    pub fn component_identifier(&self) -> &ComponentIdentifier {
        &self.component
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn set_method_auth(&mut self, method_name: &str, access_rule: AccessRule) {
        // TODO: allow setting method auth on other modules besides self
        ScryptoEnv
            .call_module_method(
                self.component.clone().into(),
                NodeModuleId::AccessRules,
                ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
                scrypto_encode(&AccessRulesSetMethodAccessRuleInput {
                    index: self.index,
                    key: AccessRuleKey::new(NodeModuleId::SELF, method_name.to_string()),
                    rule: AccessRuleEntry::AccessRule(access_rule),
                })
                .unwrap(),
            )
            .unwrap();
    }

    pub fn lock_method_auth(&mut self, method_name: &str) {
        // TODO: allow locking method auth on other modules besides self
        ScryptoEnv
            .call_module_method(
                self.component.clone().into(),
                NodeModuleId::AccessRules,
                ACCESS_RULES_SET_METHOD_MUTABILITY_IDENT,
                scrypto_encode(&AccessRulesSetMethodMutabilityInput {
                    index: self.index,
                    key: AccessRuleKey::new(NodeModuleId::SELF, method_name.to_string()),
                    mutability: AccessRule::DenyAll,
                })
                .unwrap(),
            )
            .unwrap();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, LegacyDescribe)]
pub enum ComponentIdentifier {
    RENodeId(ComponentId),
    GlobalAddress(ComponentAddress),
}

impl From<ComponentId> for ComponentIdentifier {
    fn from(value: ComponentId) -> Self {
        ComponentIdentifier::RENodeId(value)
    }
}

impl From<ComponentAddress> for ComponentIdentifier {
    fn from(value: ComponentAddress) -> Self {
        ComponentIdentifier::GlobalAddress(value)
    }
}

impl From<ComponentIdentifier> for RENodeId {
    fn from(value: ComponentIdentifier) -> Self {
        match value {
            ComponentIdentifier::RENodeId(node_id) => RENodeId::Component(node_id),
            ComponentIdentifier::GlobalAddress(component_address) => {
                RENodeId::Global(GlobalAddress::Component(component_address))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor, LegacyDescribe)]
pub enum Mutability {
    LOCKED,
    MUTABLE(AccessRule),
}

impl From<Mutability> for AccessRule {
    fn from(val: Mutability) -> Self {
        match val {
            Mutability::LOCKED => AccessRule::DenyAll,
            Mutability::MUTABLE(rule) => rule,
        }
    }
}
