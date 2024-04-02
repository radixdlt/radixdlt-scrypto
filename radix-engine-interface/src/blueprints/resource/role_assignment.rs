use crate::internal_prelude::*;
use crate::object_modules::role_assignment::ToRoleEntry;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;

use super::AccessRule;

pub const SELF_ROLE: &'static str = "_self_";
pub const OWNER_ROLE: &'static str = "_owner_";

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct MethodKey {
    pub ident: String,
}

impl MethodKey {
    pub fn new<S: ToString>(method_ident: S) -> Self {
        Self {
            ident: method_ident.to_string(),
        }
    }
}

impl From<&str> for MethodKey {
    fn from(value: &str) -> Self {
        MethodKey::new(value)
    }
}

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum MethodAccessibility {
    /// Method is accessible to all
    Public,
    /// Only outer objects have access to a given method. Currently used by Validator blueprint
    /// to only allow ConsensusManager to access some methods.
    OuterObjectOnly,
    /// Method is only accessible by any role in the role list
    RoleProtected(RoleList),
    /// Only the package this method is a part of may access this method
    OwnPackageOnly,
}

impl MethodAccessibility {
    pub fn nobody() -> Self {
        MethodAccessibility::RoleProtected(RoleList::none())
    }
}

impl<const N: usize> From<[&str; N]> for MethodAccessibility {
    fn from(value: [&str; N]) -> Self {
        MethodAccessibility::RoleProtected(value.into())
    }
}

impl From<RoleList> for MethodAccessibility {
    fn from(value: RoleList) -> Self {
        Self::RoleProtected(value)
    }
}

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct ModuleRoleKey {
    pub module: ModuleId,
    pub key: RoleKey,
}

impl ModuleRoleKey {
    pub fn new<K: Into<RoleKey>>(module: ModuleId, key: K) -> Self {
        Self {
            module,
            key: key.into(),
        }
    }
}

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    ManifestSbor,
    ScryptoCategorize,
    ScryptoDecode,
    ScryptoEncode,
)]
#[sbor(transparent)]
pub struct RoleKey {
    pub key: String,
}

impl Describe<ScryptoCustomTypeKind> for RoleKey {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::ROLE_KEY_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::role_key_type_data()
    }
}

impl From<String> for RoleKey {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for RoleKey {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl RoleKey {
    pub fn new<S: Into<String>>(key: S) -> Self {
        RoleKey { key: key.into() }
    }
}

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum OwnerRoleUpdater {
    /// Owner is fixed and cannot be updated by anyone
    None,
    /// Owner role may only be updated by the owner themself
    Owner,
    /// Owner role may be updated by the object containing the access rules.
    /// This is currently primarily used for Presecurified objects
    Object,
}

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct OwnerRoleEntry {
    pub rule: AccessRule,
    pub updater: OwnerRoleUpdater,
}

impl OwnerRoleEntry {
    pub fn new<A: Into<AccessRule>>(rule: A, updater: OwnerRoleUpdater) -> Self {
        Self {
            rule: rule.into(),
            updater,
        }
    }
}

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor, Default,
)]
#[sbor(transparent)]
pub struct RoleList {
    pub list: Vec<RoleKey>,
}

impl RoleList {
    pub fn none() -> Self {
        Self { list: vec![] }
    }

    pub fn insert<R: Into<RoleKey>>(&mut self, role: R) {
        self.list.push(role.into());
    }

    pub fn to_list(self) -> Vec<String> {
        self.list.into_iter().map(|k| k.key).collect()
    }
}

impl From<Vec<&str>> for RoleList {
    fn from(value: Vec<&str>) -> Self {
        Self {
            list: value.into_iter().map(|s| RoleKey::new(s)).collect(),
        }
    }
}

impl From<Vec<String>> for RoleList {
    fn from(value: Vec<String>) -> Self {
        Self {
            list: value.into_iter().map(|s| RoleKey::new(s)).collect(),
        }
    }
}

impl<const N: usize> From<[&str; N]> for RoleList {
    fn from(value: [&str; N]) -> Self {
        Self {
            list: value.into_iter().map(|s| RoleKey::new(s)).collect(),
        }
    }
}

/// Front end data structure for specifying owner role
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, ManifestSbor, ScryptoCategorize, ScryptoDecode, ScryptoEncode,
)]
pub enum OwnerRole {
    /// No owner role
    None,
    /// Rule protected Owner role which may not be updated
    Fixed(AccessRule),
    /// Rule protected Owner role which may only be updated by the owner themself
    Updatable(AccessRule),
}

impl Describe<ScryptoCustomTypeKind> for OwnerRole {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::OWNER_ROLE_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::owner_role_type_data()
    }
}

impl Default for OwnerRole {
    fn default() -> Self {
        OwnerRole::None
    }
}

impl Into<OwnerRoleEntry> for OwnerRole {
    fn into(self) -> OwnerRoleEntry {
        match self {
            OwnerRole::None => OwnerRoleEntry::new(AccessRule::DenyAll, OwnerRoleUpdater::None),
            OwnerRole::Fixed(rule) => OwnerRoleEntry::new(rule, OwnerRoleUpdater::None),
            OwnerRole::Updatable(rule) => OwnerRoleEntry::new(rule, OwnerRoleUpdater::Owner),
        }
    }
}

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(Default, Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct RoleAssignmentInit {
    pub data: IndexMap<RoleKey, Option<AccessRule>>,
}

impl RoleAssignmentInit {
    pub fn new() -> Self {
        RoleAssignmentInit {
            data: index_map_new(),
        }
    }

    pub fn define_role<K: Into<RoleKey>, R: ToRoleEntry>(&mut self, role: K, access_rule: R) {
        self.data.insert(role.into(), access_rule.to_role_entry());
    }
}
