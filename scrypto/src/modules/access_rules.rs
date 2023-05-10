use crate::engine::scrypto_env::ScryptoEnv;

use radix_engine_derive::*;
use radix_engine_interface::api::node_modules::auth::{AccessRulesCreateInput, ACCESS_RULES_BLUEPRINT, ACCESS_RULES_CREATE_IDENT, ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT, AccessRulesSetGroupAccessRuleInput};
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::{AccessRule, AccessRulesConfig, GroupEntry, ObjectKey};
use radix_engine_interface::constants::ACCESS_RULES_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::prelude::*;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct AccessRules(pub Own);

impl AccessRules {
    pub fn new(access_rules: AccessRulesConfig) -> Self {
        let rtn = ScryptoEnv
            .call_function(
                ACCESS_RULES_MODULE_PACKAGE,
                ACCESS_RULES_BLUEPRINT,
                ACCESS_RULES_CREATE_IDENT,
                scrypto_encode(&AccessRulesCreateInput {
                    access_rules,
                    child_blueprint_rules: btreemap!(),
                })
                .unwrap(),
            )
            .unwrap();
        let access_rules: Own = scrypto_decode(&rtn).unwrap();
        Self(access_rules)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct AttachedAccessRules(pub GlobalAddress);

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

impl From<Mutability> for GroupEntry {
    fn from(val: Mutability) -> Self {
        match val {
            Mutability::LOCKED => GroupEntry::AccessRule(AccessRule::DenyAll),
            Mutability::MUTABLE(rule) => GroupEntry::AccessRule(rule),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ActorAccessRules;

impl ActorAccessRules {
    pub fn set_group_access_rule<A: Into<GroupEntry>>(
        &self,
        name: &str,
        entry: A,
    ) {
        let _rtn = ScryptoEnv.actor_call_module_method(
            OBJECT_HANDLE_SELF,
            ObjectModuleId::AccessRules,
            ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT,
            scrypto_encode(&AccessRulesSetGroupAccessRuleInput {
                object_key: ObjectKey::SELF,
                name: name.into(),
                rule: entry.into(),
            })
                .unwrap(),
        ).unwrap();
    }
}