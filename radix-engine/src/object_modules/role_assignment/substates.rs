use crate::internal_prelude::*;
use radix_blueprint_schema_init::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
#[sbor(transparent)]
pub struct OwnerRoleSubstate {
    pub owner_role_entry: OwnerRoleEntry,
}

declare_native_blueprint_state! {
    blueprint_ident: RoleAssignment,
    blueprint_snake_case: role_assignment,
    features: {
    },
    fields: {
        owner: {
            ident: Owner,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        },
    },
    collections: {
        role_assignment: KeyValue {
            entry_ident: AccessRule,
            key_type: {
                kind: Static,
                content_type: ModuleRoleKey,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: false,
        },
    }
}

pub type RoleAssignmentOwnerV1 = OwnerRoleSubstate;
pub type RoleAssignmentAccessRuleV1 = AccessRule;
