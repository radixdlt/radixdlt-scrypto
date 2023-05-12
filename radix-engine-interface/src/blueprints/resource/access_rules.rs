use crate::api::ObjectModuleId;
use crate::blueprints::resource::*;
use crate::rule;
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::str;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use utils::btreemap;

use super::AccessRule;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct MethodKey {
    pub module_id: ObjectModuleId,
    pub ident: String,
}

impl MethodKey {
    pub fn new(module_id: ObjectModuleId, method_ident: &str) -> Self {
        Self {
            module_id,
            ident: method_ident.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct MethodEntry {
    pub authorities: Vec<String>,
}

impl MethodEntry {
    fn authority(authority: &str) -> Self {
        MethodEntry {
            authorities: vec![authority.to_string()],
        }
    }

    fn authorities(authorities: Vec<String>) -> Self {
        MethodEntry { authorities }
    }
}

impl From<String> for AccessRule {
    fn from(name: String) -> Self {
        AccessRule::Protected(AccessRuleNode::Authority(name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct MethodAuthorities {
    pub methods: BTreeMap<MethodKey, MethodEntry>,
}

impl MethodAuthorities {
    pub fn new() -> Self {
        Self {
            methods: btreemap!(),
        }
    }

    pub fn set_public(&mut self, method: &str) {
        self.methods.insert(
            MethodKey::new(ObjectModuleId::Main, method),
            MethodEntry::authority("public"),
        );
    }

    pub fn set_module_method_authority(
        &mut self,
        module_id: ObjectModuleId,
        method: &str,
        authority: &str,
    ) {
        self.methods.insert(
            MethodKey::new(module_id, method),
            MethodEntry::authority(authority),
        );
    }

    pub fn set_main_method_authority(&mut self, method: &str, authority: &str) {
        self.methods.insert(
            MethodKey::new(ObjectModuleId::Main, method),
            MethodEntry::authority(authority),
        );
    }

    pub fn set_main_method_authorities(&mut self, method: &str, authorities: Vec<String>) {
        self.methods.insert(
            MethodKey::new(ObjectModuleId::Main, method),
            MethodEntry::authorities(authorities),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct AuthorityRules {
    pub rules: BTreeMap<String, (AccessRule, AccessRule)>,
}

impl AuthorityRules {
    pub fn new() -> Self {
        Self { rules: btreemap!() }
    }

    pub fn set_authority<S: Into<String>>(
        &mut self,
        authority: S,
        rule: AccessRule,
        mutability: AccessRule,
    ) {
        self.rules.insert(authority.into(), (rule, mutability));
    }
}

pub fn package_authority_rules_from_owner_badge(
    owner_badge: &NonFungibleGlobalId,
) -> AuthorityRules {
    let mut authority_rules = AuthorityRules::new();
    authority_rules.set_authority(
        "owner",
        rule!(require(owner_badge.clone())),
        rule!(require(owner_badge.clone())),
    );
    authority_rules
}

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
