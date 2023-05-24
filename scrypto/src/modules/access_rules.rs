use crate::engine::scrypto_env::ScryptoEnv;

use crate::modules::ModuleHandle;
use crate::prelude::Attachable;
use radix_engine_derive::*;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesCreateInput, AccessRulesDefineRoleInput, AccessRulesSetRoleMutabilityInput,
    ACCESS_RULES_BLUEPRINT, ACCESS_RULES_CREATE_IDENT, ACCESS_RULES_DEFINE_ROLE_IDENT,
    ACCESS_RULES_SET_ROLE_MUTABILITY_IDENT,
};
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::{
    AccessRule, MethodKey, MethodPermission, RoleKey, RoleList, Roles,
};
use radix_engine_interface::constants::ACCESS_RULES_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::*;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccessRules(pub ModuleHandle);

impl AccessRules {
    pub fn new(
        method_permissions: BTreeMap<MethodKey, (MethodPermission, RoleList)>,
        authority_rules: Roles,
    ) -> Self {
        let rtn = ScryptoEnv
            .call_function(
                ACCESS_RULES_MODULE_PACKAGE,
                ACCESS_RULES_BLUEPRINT,
                ACCESS_RULES_CREATE_IDENT,
                scrypto_encode(&AccessRulesCreateInput {
                    method_permissions,
                    roles: authority_rules,
                    inner_blueprint_rules: btreemap!(),
                })
                .unwrap(),
            )
            .unwrap();
        let access_rules: Own = scrypto_decode(&rtn).unwrap();
        Self(ModuleHandle::Own(access_rules))
    }

    pub fn define_role<A: Into<AccessRule>>(&self, name: &str, entry: A) {
        self.call_ignore_rtn(
            ACCESS_RULES_DEFINE_ROLE_IDENT,
            &AccessRulesDefineRoleInput {
                role_key: RoleKey::new(name),
                rule: entry.into(),
            },
        );
    }

    pub fn set_role_mutability<L: Into<RoleList>>(&self, name: &str, mutability: L) {
        self.call_ignore_rtn(
            ACCESS_RULES_SET_ROLE_MUTABILITY_IDENT,
            &AccessRulesSetRoleMutabilityInput {
                role_key: RoleKey::new(name),
                mutability: mutability.into(),
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
        AccessRules::new(btreemap!(), Roles::new())
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
