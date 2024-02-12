use crate::internal_prelude::*;
use radix_engine_interface::blueprints::resource::AccessRule;

#[derive(ScryptoSbor, ScryptoEvent, Debug)]
pub struct SetRoleEvent {
    pub role_key: RoleKey,
    pub rule: AccessRule,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug)]
pub struct SetOwnerRoleEvent {
    pub rule: AccessRule,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug)]
pub struct LockOwnerRoleEvent {}
