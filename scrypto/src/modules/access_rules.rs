use crate::engine::scrypto_env::ScryptoEnv;

use crate::modules::ModuleHandle;
use crate::prelude::Attachable;
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
use radix_engine_interface::*;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccessRules(pub ModuleHandle);

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
                    inner_blueprint_rules: btreemap!(),
                })
                .unwrap(),
            )
            .unwrap();
        let access_rules: Own = scrypto_decode(&rtn).unwrap();
        Self(ModuleHandle::Own(access_rules))
    }

    pub fn set_authority_rule<A: Into<AccessRule>>(&self, name: &str, entry: A) {
        self.call_ignore_rtn(
            ACCESS_RULES_SET_AUTHORITY_RULE_IDENT,
            &AccessRulesSetAuthorityRuleInput {
                object_key: ObjectKey::SELF,
                name: name.into(),
                rule: entry.into(),
            },
        );
    }

    pub fn set_authority_rule_on_inner_blueprint<A: Into<AccessRule>>(
        &self,
        inner_blueprint: &str,
        name: &str,
        entry: A,
    ) {
        self.call_ignore_rtn(
            ACCESS_RULES_SET_AUTHORITY_RULE_IDENT,
            &AccessRulesSetAuthorityRuleInput {
                object_key: ObjectKey::inner_blueprint(inner_blueprint),
                name: name.into(),
                rule: entry.into(),
            },
        );
    }

    pub fn set_authority_mutability(&self, name: &str, mutability: AccessRule) {
        self.call_ignore_rtn(
            ACCESS_RULES_SET_AUTHORITY_MUTABILITY_IDENT,
            &AccessRulesSetAuthorityMutabilityInput {
                object_key: ObjectKey::SELF,
                name: name.to_string(),
                mutability,
            },
        );
    }
}

impl Attachable for AccessRules {
    const MODULE_ID: ObjectModuleId = ObjectModuleId::AccessRules;

    fn new(handle: ModuleHandle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &ModuleHandle {
        &self.0
    }
}

impl Default for AccessRules {
    fn default() -> Self {
        AccessRules::new(MethodAuthorities::new(), AuthorityRules::new())
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
