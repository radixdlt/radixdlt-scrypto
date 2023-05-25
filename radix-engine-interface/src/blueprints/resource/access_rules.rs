use crate::api::ObjectModuleId;
use crate::blueprints::resource::*;
use crate::rule;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use sbor::rust::collections::BTreeMap;
use sbor::rust::str;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use utils::btreemap;

use super::AccessRule;

pub const SELF_ROLE: &'static str = "self";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct FnKey {
    pub blueprint: String,
    pub ident: String,
}

impl FnKey {
    pub fn new(blueprint: String, ident: String) -> Self {
        Self { blueprint, ident }
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum ObjectKey {
    SELF,
    InnerBlueprint(String),
}

impl ObjectKey {
    pub fn inner_blueprint(name: &str) -> Self {
        ObjectKey::InnerBlueprint(name.to_string())
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct MethodKey {
    pub module_id: ObjectModuleId,
    pub ident: String,
}

impl MethodKey {
    pub fn new<S: ToString>(module_id: ObjectModuleId, method_ident: S) -> Self {
        Self {
            module_id,
            ident: method_ident.to_string(),
        }
    }

    pub fn metadata<S: ToString>(method_ident: S) -> Self {
        Self {
            module_id: ObjectModuleId::Metadata,
            ident: method_ident.to_string(),
        }
    }

    pub fn main<S: ToString>(method_ident: S) -> Self {
        Self {
            module_id: ObjectModuleId::Main,
            ident: method_ident.to_string(),
        }
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct MethodEntry {
    pub permission: MethodPermission,
    pub mutable: RoleList,
}

impl MethodEntry {
    pub fn disabled() -> Self {
        Self {
            permission: MethodPermission::nobody(),
            mutable: RoleList::none(),
        }
    }

    pub fn new<P: Into<MethodPermission>, M: Into<RoleList>>(permission: P, mutable: M) -> Self {
        Self {
            permission: permission.into(),
            mutable: mutable.into(),
        }
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum MethodPermission {
    Public,
    Protected(RoleList),
}

impl MethodPermission {
    pub fn nobody() -> Self {
        MethodPermission::Protected(RoleList::none())
    }
}

impl<const N: usize> From<[&str; N]> for MethodPermission {
    fn from(value: [&str; N]) -> Self {
        MethodPermission::Protected(value.into())
    }
}

impl From<RoleList> for MethodPermission {
    fn from(value: RoleList) -> Self {
        Self::Protected(value)
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum AttachedModule {
    Metadata,
    Royalty,
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
pub struct RoleEntry {
    pub rule: AccessRule,
    pub mutable: RoleList,
}

impl RoleEntry {
    pub fn new<A: Into<AccessRule>, M: Into<RoleList>>(rule: A, mutable: M) -> Self {
        Self {
            rule: rule.into(),
            mutable: mutable.into(),
        }
    }

    pub fn immutable<A: Into<AccessRule>>(rule: A) -> Self {
        Self {
            rule: rule.into(),
            mutable: RoleList::none(),
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
}

impl From<Vec<&str>> for RoleList {
    fn from(value: Vec<&str>) -> Self {
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

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor, ManifestSbor)]
pub enum OwnerRule {
    None,
    Fixed(AccessRule),
    Updateable(AccessRule),
}

impl OwnerRule {
    pub fn to_role_entry(self, owner_role_name: &str) -> RoleEntry {
        match self {
            OwnerRule::Fixed(rule) => RoleEntry::new(rule, RoleList::none()),
            OwnerRule::Updateable(rule) => RoleEntry::new(rule, [owner_role_name]),
            OwnerRule::None => RoleEntry::new(AccessRule::DenyAll, RoleList::none()),
        }
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct Roles {
    pub rules: BTreeMap<RoleKey, RoleEntry>,
}

impl Roles {
    pub fn new() -> Self {
        Self { rules: btreemap!() }
    }

    pub fn new_with_owner_authority(owner_badge: &NonFungibleGlobalId) -> Roles {
        let mut authority_rules = Roles::new();
        authority_rules.define_role(
            "owner",
            RoleEntry::new(rule!(require(owner_badge.clone())), ["owner"]),
        );
        authority_rules
    }

    pub fn define_role<K: Into<RoleKey>>(
        &mut self,
        authority: K,
        entry: RoleEntry,
    ) {
        self.rules
            .insert(authority.into(), entry);
    }
}

// TODO: Remove?
pub fn resource_access_rules_from_owner_badge(
    owner_badge: &NonFungibleGlobalId,
) -> BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)> {
    let mut access_rules = BTreeMap::new();
    access_rules.insert(
        ResourceMethodAuthKey::Withdraw,
        (AccessRule::AllowAll, rule!(require(owner_badge.clone()))),
    );
    access_rules.insert(
        ResourceMethodAuthKey::Deposit,
        (AccessRule::AllowAll, rule!(require(owner_badge.clone()))),
    );
    access_rules.insert(
        ResourceMethodAuthKey::Recall,
        (AccessRule::DenyAll, rule!(require(owner_badge.clone()))),
    );
    access_rules.insert(
        Mint,
        (AccessRule::DenyAll, rule!(require(owner_badge.clone()))),
    );
    access_rules.insert(
        Burn,
        (AccessRule::DenyAll, rule!(require(owner_badge.clone()))),
    );
    access_rules.insert(
        UpdateNonFungibleData,
        (
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        ),
    );
    access_rules.insert(
        UpdateMetadata,
        (
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        ),
    );
    access_rules
}
