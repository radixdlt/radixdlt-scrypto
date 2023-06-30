use crate::engine::scrypto_env::ScryptoEnv;

use crate::modules::ModuleHandle;
use crate::prelude::Attachable;
use radix_engine_derive::*;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesCreateInput, AccessRulesGetRoleInput, AccessRulesLockOwnerRoleInput,
    AccessRulesLockRoleInput, AccessRulesSetAndLockOwnerRoleInput, AccessRulesSetAndLockRoleInput,
    AccessRulesSetOwnerRoleInput, AccessRulesSetRoleInput, ACCESS_RULES_BLUEPRINT,
    ACCESS_RULES_CREATE_IDENT, ACCESS_RULES_GET_ROLE_IDENT, ACCESS_RULES_LOCK_OWNER_ROLE_IDENT,
    ACCESS_RULES_LOCK_ROLE_IDENT, ACCESS_RULES_SET_AND_LOCK_OWNER_ROLE_IDENT,
    ACCESS_RULES_SET_AND_LOCK_ROLE_IDENT, ACCESS_RULES_SET_OWNER_ROLE_IDENT,
    ACCESS_RULES_SET_ROLE_IDENT,
};
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::{AccessRule, OwnerRole, RoleKey, RolesInit};
use radix_engine_interface::constants::ACCESS_RULES_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::*;
use sbor::rust::prelude::*;

pub trait HasAccessRules {
    fn set_owner_role<A: Into<AccessRule>>(&self, rule: A);
    fn lock_owner_role<A: Into<AccessRule>>(&self);
    fn set_and_lock_owner_role<A: Into<AccessRule>>(&self, rule: A);
    fn set_role<A: Into<AccessRule>>(&self, name: &str, rule: A);
    fn get_role(&self, name: &str) -> Option<AccessRule>;
    fn lock_role(&self, name: &str);
    fn set_and_lock_role<A: Into<AccessRule>>(&self, name: &str, rule: A);
    fn set_metadata_role<A: Into<AccessRule>>(&self, name: &str, rule: A);
    fn lock_metadata_role(&self, name: &str);
    fn set_and_lock_metadata_role<A: Into<AccessRule>>(&self, name: &str, rule: A);
    fn set_component_royalties_role<A: Into<AccessRule>>(&self, name: &str, rule: A);
    fn lock_component_royalties_role(&self, name: &str);
    fn set_and_lock_component_royalties_role<A: Into<AccessRule>>(&self, name: &str, rule: A);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccessRules(pub ModuleHandle);

impl AccessRules {
    pub fn new(owner_role: OwnerRole, roles: BTreeMap<ObjectModuleId, RolesInit>) -> Self {
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

    pub fn set_owner_role<A: Into<AccessRule>>(&self, rule: A) {
        self.call_ignore_rtn(
            ACCESS_RULES_SET_OWNER_ROLE_IDENT,
            &AccessRulesSetOwnerRoleInput { rule: rule.into() },
        );
    }

    pub fn lock_owner_role(&self) {
        self.call_ignore_rtn(
            ACCESS_RULES_LOCK_OWNER_ROLE_IDENT,
            &AccessRulesLockOwnerRoleInput {},
        );
    }

    pub fn set_and_lock_owner_role<A: Into<AccessRule>>(&self, rule: A) {
        self.call_ignore_rtn(
            ACCESS_RULES_SET_AND_LOCK_OWNER_ROLE_IDENT,
            &AccessRulesSetAndLockOwnerRoleInput { rule: rule.into() },
        );
    }

    fn internal_set_role<A: Into<AccessRule>>(&self, module: ObjectModuleId, name: &str, rule: A) {
        self.call_ignore_rtn(
            ACCESS_RULES_SET_ROLE_IDENT,
            &AccessRulesSetRoleInput {
                module,
                role_key: RoleKey::new(name),
                rule: rule.into(),
            },
        );
    }

    fn internal_get_role(&self, module: ObjectModuleId, name: &str) -> Option<AccessRule> {
        self.call(
            ACCESS_RULES_GET_ROLE_IDENT,
            &AccessRulesGetRoleInput {
                module,
                role_key: RoleKey::new(name),
            },
        )
    }

    fn internal_lock_role(&self, module: ObjectModuleId, name: &str) {
        self.call_ignore_rtn(
            ACCESS_RULES_LOCK_ROLE_IDENT,
            &AccessRulesLockRoleInput {
                module,
                role_key: RoleKey::new(name),
            },
        );
    }

    fn internal_set_and_lock_role<A: Into<AccessRule>>(
        &self,
        module: ObjectModuleId,
        name: &str,
        rule: A,
    ) {
        self.call_ignore_rtn(
            ACCESS_RULES_SET_AND_LOCK_ROLE_IDENT,
            &AccessRulesSetAndLockRoleInput {
                module,
                role_key: RoleKey::new(name),
                rule: rule.into(),
            },
        );
    }

    pub fn set_role<A: Into<AccessRule>>(&self, name: &str, rule: A) {
        self.internal_set_role(ObjectModuleId::Main, name, rule);
    }

    pub fn get_role(&self, name: &str) -> Option<AccessRule> {
        self.internal_get_role(ObjectModuleId::Main, name)
    }

    pub fn lock_role(&self, name: &str) {
        self.internal_lock_role(ObjectModuleId::Main, name);
    }

    pub fn set_and_lock_role<A: Into<AccessRule>>(&self, name: &str, rule: A) {
        self.internal_set_and_lock_role(ObjectModuleId::Main, name, rule);
    }

    pub fn set_metadata_role<A: Into<AccessRule>>(&self, name: &str, rule: A) {
        self.internal_set_role(ObjectModuleId::Metadata, name, rule);
    }

    pub fn get_metadata_role<A: Into<AccessRule>>(&self, name: &str) {
        self.internal_get_role(ObjectModuleId::Metadata, name);
    }

    pub fn lock_metadata_role(&self, name: &str) {
        self.internal_lock_role(ObjectModuleId::Metadata, name);
    }

    pub fn set_and_lock_metadata_role<A: Into<AccessRule>>(&self, name: &str, rule: A) {
        self.internal_set_and_lock_role(ObjectModuleId::Metadata, name, rule);
    }

    pub fn set_component_royalties_role<A: Into<AccessRule>>(&self, name: &str, rule: A) {
        self.internal_set_role(ObjectModuleId::Royalty, name, rule);
    }

    pub fn get_component_royalties_role<A: Into<AccessRule>>(&self, name: &str) {
        self.internal_get_role(ObjectModuleId::Royalty, name);
    }

    pub fn lock_component_royalties_role(&self, name: &str) {
        self.internal_lock_role(ObjectModuleId::Royalty, name);
    }

    pub fn set_and_lock_component_royalties_role<A: Into<AccessRule>>(&self, name: &str, rule: A) {
        self.internal_set_and_lock_role(ObjectModuleId::Royalty, name, rule);
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
