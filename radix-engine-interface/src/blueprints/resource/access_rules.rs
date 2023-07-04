use crate::api::node_modules::auth::ToRoleEntry;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_interface::api::ObjectModuleId;
use sbor::rust::str;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use std::collections::BTreeMap;

use super::AccessRule;

pub const SELF_ROLE: &'static str = "_self_";
pub const OWNER_ROLE: &'static str = "_owner_";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
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

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
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

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct ModuleRoleKey {
    pub module: ObjectModuleId,
    pub key: RoleKey,
}

impl ModuleRoleKey {
    pub fn new<K: Into<RoleKey>>(module: ObjectModuleId, key: K) -> Self {
        Self {
            module,
            key: key.into(),
        }
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct RoleKey {
    pub key: String,
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

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum OwnerRoleUpdater {
    /// Owner is fixed and cannot be updated by anyone
    None,
    /// Owner role may only be updated by the owner themself
    Owner,
    /// Owner role may be updated by the object containing the access rules.
    /// This is currently primarily used for Presecurified objects
    Object,
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
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

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
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
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor, ManifestSbor)]
pub enum OwnerRole {
    /// No owner role
    None,
    /// Rule protected Owner role which may not be updated
    Fixed(AccessRule),
    /// Rule protected Owner role which may only be updated by the owner themself
    Updatable(AccessRule),
    /// Rule protected Owner role which may only be updated by the object
    /// containing the access rules.
    /// This is currently primarily used for Presecurified objects
    UpdatableByObject(AccessRule),
}

impl OwnerRole {
    pub fn to_entry(self) -> OwnerRoleEntry {
        match self {
            OwnerRole::None => OwnerRoleEntry::new(AccessRule::DenyAll, OwnerRoleUpdater::None),
            OwnerRole::Fixed(rule) => OwnerRoleEntry::new(rule, OwnerRoleUpdater::None),
            OwnerRole::Updatable(rule) => OwnerRoleEntry::new(rule, OwnerRoleUpdater::Owner),
            OwnerRole::UpdatableByObject(rule) => {
                OwnerRoleEntry::new(rule, OwnerRoleUpdater::Object)
            }
        }
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct RolesInit {
    pub data: BTreeMap<RoleKey, Option<AccessRule>>,
}

impl RolesInit {
    pub fn new() -> Self {
        RolesInit {
            data: BTreeMap::new(),
        }
    }

    pub fn define_role<K: Into<RoleKey>, R: ToRoleEntry>(&mut self, role: K, access_rule: R) {
        self.data.insert(role.into(), access_rule.to_role_entry());
    }
}
