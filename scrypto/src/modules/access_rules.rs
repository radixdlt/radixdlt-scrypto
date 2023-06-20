use crate::engine::scrypto_env::ScryptoEnv;

use crate::modules::ModuleHandle;
use crate::prelude::Attachable;
use radix_engine_derive::*;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesCreateInput, AccessRulesUpdateRoleInput, ACCESS_RULES_BLUEPRINT,
    ACCESS_RULES_CREATE_IDENT, ACCESS_RULES_UPDATE_ROLE_IDENT,
};
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::{
    AccessRule, OwnerRole, RoleKey, RoleList, Roles,
};
use radix_engine_interface::constants::ACCESS_RULES_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::*;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccessRules(pub ModuleHandle);

impl AccessRules {
    pub fn new(owner_role: OwnerRole, roles: BTreeMap<ObjectModuleId, Roles>) -> Self {
        let rtn = ScryptoEnv
            .call_function(
                ACCESS_RULES_MODULE_PACKAGE,
                ACCESS_RULES_BLUEPRINT,
                ACCESS_RULES_CREATE_IDENT,
                scrypto_encode(&AccessRulesCreateInput { owner_role, roles }).unwrap(),
            )
            .unwrap();
        let access_rules: Own = scrypto_decode(&rtn).unwrap();
        Self(ModuleHandle::Own(access_rules))
    }

    pub fn update_role_rule<A: Into<AccessRule>>(&self, name: &str, entry: A) {
        self.call_ignore_rtn(
            ACCESS_RULES_UPDATE_ROLE_IDENT,
            &AccessRulesUpdateRoleInput {
                module: ObjectModuleId::Main,
                role_key: RoleKey::new(name),
                rule: Some(entry.into()),
                mutability: None,
            },
        );
    }

    pub fn update_role_mutability<L: Into<RoleList>>(&self, name: &str, mutability: L) {
        self.call_ignore_rtn(
            ACCESS_RULES_UPDATE_ROLE_IDENT,
            &AccessRulesUpdateRoleInput {
                module: ObjectModuleId::Main,
                role_key: RoleKey::new(name),
                rule: None,
                mutability: Some((mutability.into(), true)),
            },
        );
    }

    pub fn update_metadata_role_rule<A: Into<AccessRule>>(&self, name: &str, entry: A) {
        self.call_ignore_rtn(
            ACCESS_RULES_UPDATE_ROLE_IDENT,
            &AccessRulesUpdateRoleInput {
                module: ObjectModuleId::Metadata,
                role_key: RoleKey::new(name),
                rule: Some(entry.into()),
                mutability: None,
            },
        );
    }

    pub fn update_metadata_role_mutability<L: Into<RoleList>>(&self, name: &str, mutability: L) {
        self.call_ignore_rtn(
            ACCESS_RULES_UPDATE_ROLE_IDENT,
            &AccessRulesUpdateRoleInput {
                module: ObjectModuleId::Metadata,
                role_key: RoleKey::new(name),
                rule: None,
                mutability: Some((mutability.into(), true)),
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
