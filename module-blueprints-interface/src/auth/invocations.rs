use radix_engine_common::prelude::*;

pub const ROLE_ASSIGNMENT_CREATE_IDENT: &str = "create";

#[cfg_attr(
    feature = "radix_engine_fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct RoleAssignmentCreateInput {
    pub owner_role: OwnerRoleEntry,
    pub roles: IndexMap<ModuleId, RoleAssignmentInit>,
}

pub type RoleAssignmentCreateOutput = Own;

pub const ROLE_ASSIGNMENT_SET_IDENT: &str = "set";

#[cfg_attr(
    feature = "radix_engine_fuzzing",
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

pub type RoleAssignmentSetOutput = ();

pub const ROLE_ASSIGNMENT_SET_OWNER_IDENT: &str = "set_owner";

#[cfg_attr(
    feature = "radix_engine_fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct RoleAssignmentSetOwnerInput {
    pub rule: AccessRule,
}

pub type RoleAssignmentSetOwnerOutput = ();

pub const ROLE_ASSIGNMENT_LOCK_OWNER_IDENT: &str = "lock_owner";

#[cfg_attr(
    feature = "radix_engine_fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct RoleAssignmentLockOwnerInput {}

pub type RoleAssingmentLockOwnerOutput = ();

pub const ROLE_ASSIGNMENT_GET_IDENT: &str = "get";

#[cfg_attr(
    feature = "radix_engine_fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct RoleAssignmentGetInput {
    pub module: ModuleId,
    pub role_key: RoleKey,
}

pub type RoleAssignmentGetOutput = Option<AccessRule>;
