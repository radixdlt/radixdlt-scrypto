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

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct ManifestOwnerRoleEntry {
    pub rule: ManifestAccessRule,
    pub updater: OwnerRoleUpdater,
}

impl From<OwnerRoleEntry> for ManifestOwnerRoleEntry {
    fn from(value: OwnerRoleEntry) -> Self {
        Self {
            rule: value.rule.into(),
            updater: value.updater,
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
    Default,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    ManifestSbor,
    ScryptoCategorize,
    ScryptoDecode,
    ScryptoEncode,
)]
pub enum OwnerRole {
    /// No owner role
    #[default]
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

impl From<OwnerRole> for OwnerRoleEntry {
    fn from(val: OwnerRole) -> Self {
        match val {
            OwnerRole::None => OwnerRoleEntry::new(AccessRule::DenyAll, OwnerRoleUpdater::None),
            OwnerRole::Fixed(rule) => OwnerRoleEntry::new(rule, OwnerRoleUpdater::None),
            OwnerRole::Updatable(rule) => OwnerRoleEntry::new(rule, OwnerRoleUpdater::Owner),
        }
    }
}

/// An un-typed alternative to [`OwnerRole`] that implements [`ManifestSbor`]. This is designed to
/// be used in manifest invocations.
///
/// When decoding into a [`ManifestOwnerRole`] the SBOR payload is checked to ensure that it's an
/// enum. No other checks are performed beyond that as checking that the variant is actually inline
/// with what [`OwnerRole`] expects or that the values are the same.
///
/// This is a transparent wrapper around a semi-typed [`ManifestValue`] that's restricted to enums
/// only through the use of [`EnumVariantValue`].
#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor)]
#[sbor(transparent)]
pub struct ManifestOwnerRole(EnumVariantValue<ManifestCustomValueKind, ManifestCustomValue>);

impl From<OwnerRole> for ManifestOwnerRole {
    fn from(value: OwnerRole) -> Self {
        manifest_decode(&manifest_encode(&value).unwrap())
            .map(Self)
            .unwrap()
    }
}

impl Describe<ScryptoCustomTypeKind> for ManifestOwnerRole {
    const TYPE_ID: RustTypeId = <OwnerRole as Describe<ScryptoCustomTypeKind>>::TYPE_ID;

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        <OwnerRole as Describe<ScryptoCustomTypeKind>>::type_data()
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

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(Default, Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct ManifestRoleAssignmentInit {
    pub data: IndexMap<RoleKey, Option<ManifestAccessRule>>,
}

impl ManifestRoleAssignmentInit {
    pub fn new() -> Self {
        ManifestRoleAssignmentInit {
            data: index_map_new(),
        }
    }

    pub fn define_role<K: Into<RoleKey>, R: ToRoleEntry>(&mut self, role: K, access_rule: R) {
        self.data
            .insert(role.into(), access_rule.to_role_entry().map(Into::into));
    }
}

impl From<RoleAssignmentInit> for ManifestRoleAssignmentInit {
    fn from(value: RoleAssignmentInit) -> Self {
        Self {
            data: value
                .data
                .into_iter()
                .map(|(key, value)| (key, value.map(Into::into)))
                .collect(),
        }
    }
}
