use crate::engine::scrypto_env::ScryptoEnv;

use radix_engine_derive::*;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesCreateInput, AccessRulesSetMethodAccessRuleInput,
    AccessRulesSetMethodMutabilityInput, ACCESS_RULES_BLUEPRINT, ACCESS_RULES_CREATE_IDENT,
    ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT, ACCESS_RULES_SET_METHOD_MUTABILITY_IDENT,
};
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::{
    AccessRule, AccessRuleEntry, AccessRulesConfig, MethodKey,
};
use radix_engine_interface::constants::ACCESS_RULES_PACKAGE;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::*;
use sbor::rust::prelude::*;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct AccessRules(pub ObjectId);

impl AccessRules {
    pub fn new(access_rules: AccessRulesConfig) -> Self {
        let rtn = ScryptoEnv
            .call_function(
                ACCESS_RULES_PACKAGE,
                ACCESS_RULES_BLUEPRINT,
                ACCESS_RULES_CREATE_IDENT,
                scrypto_encode(&AccessRulesCreateInput { access_rules }).unwrap(),
            )
            .unwrap();
        let access_rules: Own = scrypto_decode(&rtn).unwrap();
        Self(access_rules.id())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct AttachedAccessRules(pub Address);

impl AttachedAccessRules {
    pub fn set_method_auth(&mut self, method_name: &str, access_rule: AccessRule) {
        // TODO: allow setting method auth on other modules besides self
        ScryptoEnv
            .call_module_method(
                self.0.clone().into(),
                NodeModuleId::AccessRules,
                ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
                scrypto_encode(&AccessRulesSetMethodAccessRuleInput {
                    key: MethodKey::new(NodeModuleId::SELF, method_name),
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
                self.0.clone().into(),
                NodeModuleId::AccessRules,
                ACCESS_RULES_SET_METHOD_MUTABILITY_IDENT,
                scrypto_encode(&AccessRulesSetMethodMutabilityInput {
                    key: MethodKey::new(NodeModuleId::SELF, method_name),
                    mutability: AccessRuleEntry::AccessRule(AccessRule::DenyAll),
                })
                .unwrap(),
            )
            .unwrap();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
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

impl From<Mutability> for AccessRuleEntry {
    fn from(val: Mutability) -> Self {
        match val {
            Mutability::LOCKED => AccessRuleEntry::AccessRule(AccessRule::DenyAll),
            Mutability::MUTABLE(rule) => AccessRuleEntry::AccessRule(rule),
        }
    }
}
