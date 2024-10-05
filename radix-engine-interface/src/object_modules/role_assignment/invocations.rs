use crate::api::ModuleId;
use crate::blueprints::resource::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use radix_common::prelude::*;
use sbor::rust::fmt::Debug;

pub const ROLE_ASSIGNMENT_BLUEPRINT: &str = "RoleAssignment";

pub const ROLE_ASSIGNMENT_CREATE_IDENT: &str = "create";

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct RoleAssignmentCreateInput {
    pub owner_role: OwnerRoleEntry,
    pub roles: IndexMap<ModuleId, RoleAssignmentInit>,
}

pub type RoleAssignmentCreateManifestInput = RoleAssignmentCreateInput;

pub type RoleAssignmentCreateOutput = Own;

pub const ROLE_ASSIGNMENT_SET_IDENT: &str = "set";

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct RoleAssignmentSetInput {
    pub module: ModuleId,
    pub role_key: RoleKey,
    pub rule: AccessRule,
}

pub type RoleAssignmentSetManifestInput = RoleAssignmentSetInput;

pub type RoleAssignmentSetOutput = ();

pub const ROLE_ASSIGNMENT_SET_OWNER_IDENT: &str = "set_owner";

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct RoleAssignmentSetOwnerInput {
    pub rule: AccessRule,
}

pub type RoleAssignmentSetOwnerManifestInput = RoleAssignmentSetOwnerInput;

pub type RoleAssignmentSetOwnerOutput = ();

pub const ROLE_ASSIGNMENT_LOCK_OWNER_IDENT: &str = "lock_owner";

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct RoleAssignmentLockOwnerInput {}

pub type RoleAssignmentLockOwnerManifestInput = RoleAssignmentLockOwnerInput;

pub type RoleAssignmentLockOwnerOutput = ();

pub const ROLE_ASSIGNMENT_GET_IDENT: &str = "get";

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct RoleAssignmentGetInput {
    pub module: ModuleId,
    pub role_key: RoleKey,
}

pub type RoleAssignmentGetManifestInput = RoleAssignmentGetInput;

pub type RoleAssignmentGetOutput = Option<AccessRule>;

// Part of the Bottlenose protocol update with the role assignment blueprint extension.
pub const ROLE_ASSIGNMENT_GET_OWNER_ROLE_IDENT: &str = "get_owner_role";

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct RoleAssignmentGetOwnerRoleInput;

pub type RoleAssignmentGetOwnerRoleManifestInput = RoleAssignmentGetOwnerRoleInput;

pub type RoleAssignmentGetOwnerRoleOutput = OwnerRoleEntry;

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
        $role_struct::<$crate::object_modules::role_assignment::RoleDefinition> {
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
        $crate::object_modules::role_assignment::ToRoleEntry::to_role_entry($rule)
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
