use crate::types::*;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use sbor::rust::fmt::Debug;
use sbor::rust::prelude::*;

pub trait ToRoleEntry {
    fn to_role_entry(self) -> Option<AccessRule>;
}

impl ToRoleEntry for AccessRule {
    fn to_role_entry(self) -> Option<AccessRule> {
        Some(self)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum FallToOwner {
    OWNER,
}

impl ToRoleEntry for FallToOwner {
    fn to_role_entry(self) -> Option<AccessRule> {
        match self {
            FallToOwner::OWNER => None,
        }
    }
}

impl ToRoleEntry for Option<AccessRule> {
    fn to_role_entry(self) -> Option<AccessRule> {
        self
    }
}

pub type RoleDefinition = Option<AccessRule>;

#[macro_export]
macro_rules! internal_roles {
    ($role_struct:ident, $($role:ident => $rule:expr;)* ) => ({
        let method_roles = $crate::internal_roles_struct!($role_struct, $($role => $rule;)*);

        let mut roles = $crate::blueprints::resource::RoleAssignmentInit::new();
        for (name, entry) in method_roles.list() {
            roles.define_role(name, entry);
        }

        roles
    });
}

#[macro_export]
macro_rules! internal_roles_struct {
    ($role_struct:ident, $($role:ident => $rule:expr;)* ) => ({
        $role_struct::<$crate::types::RoleDefinition> {
            $(
                $role: {
                    $crate::role_definition_entry!($rule)
                }
            ),*
        }
    });
}

#[macro_export]
macro_rules! role_definition_entry {
    ($rule:expr) => {{
        $crate::types::ToRoleEntry::to_role_entry($rule)
    }};
}

#[macro_export]
macro_rules! roles_init_set_entry {
    ($roles:expr, $key:expr, $value:expr) => {{
        $roles.define_role($key, $value);
    }};
}

#[macro_export]
macro_rules! roles_init {
    () => ({
        RoleAssignmentInit::new()
    });
    ( $($key:expr => $value:expr;)* ) => ({
        let mut roles_init = RoleAssignmentInit::new();
        $(
            $crate::roles_init_set_entry!(roles_init, $key, $value);
        )*
        roles_init
    });
}
