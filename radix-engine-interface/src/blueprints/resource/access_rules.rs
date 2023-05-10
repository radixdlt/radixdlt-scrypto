use crate::api::ObjectModuleId;
use crate::blueprints::package::PACKAGE_CLAIM_ROYALTY_IDENT;
use crate::blueprints::package::PACKAGE_SET_ROYALTY_CONFIG_IDENT;
use crate::blueprints::resource::*;
use crate::rule;
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::str;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;

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
    ChildBlueprint(String),
}

impl ObjectKey {
    pub fn child_blueprint(name: &str) -> Self {
        ObjectKey::ChildBlueprint(name.to_string())
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

pub struct AuthorityUtil;

impl AuthorityUtil {
    pub fn authority(name: &str) -> AccessRule {
        AccessRule::Protected(AccessRuleNode::Authority(name.to_string()))
    }
}

impl From<String> for AccessRule {
    fn from(value: String) -> Self {
        AuthorityUtil::authority(value.as_str())
    }
}

/// Method authorization rules for a component
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct AccessRulesConfig {
    pub direct_methods: BTreeMap<MethodKey, MethodEntry>,
    pub methods: BTreeMap<MethodKey, MethodEntry>,

    pub authorities: BTreeMap<String, AccessRule>,
    pub mutability: BTreeMap<String, AccessRule>,
}

impl AccessRulesConfig {
    pub fn new() -> Self {
        Self {
            direct_methods: BTreeMap::new(),
            methods: BTreeMap::new(),
            authorities: BTreeMap::new(),
            mutability: BTreeMap::new(),
        }
    }

    pub fn get_authority_mutability(&self, key: &str) -> AccessRule {
        match self.mutability.get(key) {
            None => AccessRule::DenyAll,
            Some(entry) => entry.clone(),
        }
    }

    pub fn set_authority_access_rule<E: Into<AccessRule>>(
        &mut self,
        group_key: String,
        access_rule_entry: E,
    ) {
        self.authorities.insert(group_key, access_rule_entry.into());
    }

    pub fn set_authority_mutability<M: Into<AccessRule>>(
        &mut self,
        key: String,
        method_auth: M,
    ) {
        self.mutability.insert(key, method_auth.into());
    }

    pub fn set_authority_access_rule_and_mutability<
        E: Into<AccessRule>,
        M: Into<AccessRule>,
    >(
        &mut self,
        authority: &str,
        access_rule: E,
        mutability: M,
    ) {
        self.authorities
            .insert(authority.to_string(), access_rule.into());
        self.mutability
            .insert(authority.to_string(), mutability.into());
    }

    pub fn set_public(&mut self, key: MethodKey) {
        self.set_group(key, "public");
    }

    pub fn set_group(&mut self, key: MethodKey, group: &str) {
        self.methods.insert(key.clone(), MethodEntry::authority(group));
    }

    pub fn set_groups(&mut self, key: MethodKey, groups: Vec<String>) {
        self.methods
            .insert(key.clone(), MethodEntry::authorities(groups));
    }

    pub fn set_main_method_group(&mut self, method: &str, group: &str) {
        let key = MethodKey::new(ObjectModuleId::Main, method);
        self.methods.insert(key.clone(), MethodEntry::authority(group));
    }

    pub fn set_direct_access_group(&mut self, key: MethodKey, group: &str) {
        self.direct_methods
            .insert(key.clone(), MethodEntry::authority(group));
    }
}

pub fn package_access_rules_from_owner_badge(
    owner_badge: &NonFungibleGlobalId,
) -> AccessRulesConfig {
    let mut access_rules = AccessRulesConfig::new();
    access_rules.set_authority_access_rule_and_mutability(
        "update_metadata",
        rule!(require(owner_badge.clone())),
        rule!(require(owner_badge.clone())),
    );
    access_rules.set_authority_access_rule_and_mutability(
        "royalty",
        rule!(require(owner_badge.clone())),
        rule!(require(owner_badge.clone())),
    );
    access_rules.set_group(
        MethodKey::new(ObjectModuleId::Main, PACKAGE_SET_ROYALTY_CONFIG_IDENT),
        "royalty",
    );
    access_rules.set_group(
        MethodKey::new(ObjectModuleId::Main, PACKAGE_CLAIM_ROYALTY_IDENT),
        "royalty",
    );
    access_rules
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
