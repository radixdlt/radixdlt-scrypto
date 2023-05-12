use crate::engine::scrypto_env::ScryptoEnv;

use radix_engine_derive::*;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesCreateInput, AccessRulesSetAuthorityMutabilityInput,
    AccessRulesSetAuthorityRuleInput, ACCESS_RULES_BLUEPRINT, ACCESS_RULES_CREATE_IDENT,
    ACCESS_RULES_SET_AUTHORITY_MUTABILITY_IDENT, ACCESS_RULES_SET_AUTHORITY_RULE_IDENT,
};
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::{
    AccessRule, AuthorityRules, MethodAuthorities, ObjectKey,
};
use radix_engine_interface::constants::ACCESS_RULES_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::prelude::*;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct AccessRules(pub Own);

impl AccessRules {
    pub fn new(method_authorities: MethodAuthorities, authority_rules: AuthorityRules) -> Self {
        let rtn = ScryptoEnv
            .call_function(
                ACCESS_RULES_MODULE_PACKAGE,
                ACCESS_RULES_BLUEPRINT,
                ACCESS_RULES_CREATE_IDENT,
                scrypto_encode(&AccessRulesCreateInput {
                    method_authorities,
                    authority_rules,
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ActorAccessRules;

impl ActorAccessRules {
    pub fn set_group_access_rule<A: Into<AccessRule>>(&self, name: &str, entry: A) {
        let _rtn = ScryptoEnv
            .actor_call_module_method(
                OBJECT_HANDLE_SELF,
                ObjectModuleId::AccessRules,
                ACCESS_RULES_SET_AUTHORITY_RULE_IDENT,
                scrypto_encode(&AccessRulesSetAuthorityRuleInput {
                    object_key: ObjectKey::SELF,
                    name: name.into(),
                    rule: entry.into(),
                })
                .unwrap(),
            )
            .unwrap();
    }

    pub fn set_group_mutability(&self, name: &str, mutability: AccessRule) {
        let _rtn = ScryptoEnv
            .actor_call_module_method(
                OBJECT_HANDLE_SELF,
                ObjectModuleId::AccessRules,
                ACCESS_RULES_SET_AUTHORITY_MUTABILITY_IDENT,
                scrypto_encode(&AccessRulesSetAuthorityMutabilityInput {
                    object_key: ObjectKey::SELF,
                    name: name.to_string(),
                    mutability,
                })
                .unwrap(),
            )
            .unwrap();
    }
}
