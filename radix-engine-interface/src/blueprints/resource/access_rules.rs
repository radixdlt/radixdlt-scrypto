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
    pub groups: Vec<String>,
}

impl MethodEntry {
    fn group(group: &str) -> Self {
        MethodEntry {
            groups: vec![group.to_string()],
        }
    }

    fn groups(groups: Vec<String>) -> Self {
        MethodEntry { groups }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum AuthorityEntry {
    AccessRule(AccessRule),
    Group(String),
    Groups(Vec<String>),
}

impl AuthorityEntry {
    pub fn group(name: &str) -> Self {
        Self::Group(name.to_string())
    }
}

impl From<AccessRule> for AuthorityEntry {
    fn from(value: AccessRule) -> Self {
        AuthorityEntry::AccessRule(value)
    }
}

impl From<String> for AuthorityEntry {
    fn from(value: String) -> Self {
        AuthorityEntry::Group(value)
    }
}

/// Method authorization rules for a component
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct AccessRulesConfig {
    direct_methods: BTreeMap<MethodKey, MethodEntry>,
    methods: BTreeMap<MethodKey, MethodEntry>,

    authorities: BTreeMap<String, AuthorityEntry>,
    mutability: BTreeMap<String, AuthorityEntry>,
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

    pub fn get_access_rules(&self, is_direct_access: bool, key: &MethodKey) -> Vec<AccessRule> {
        let auth = if is_direct_access {
            &self.direct_methods
        } else {
            &self.methods
        };
        match auth.get(key) {
            None => vec![], // FIXME: This should really be DenyAll but leave it as AllowAll for now until scrypto side fixed
            Some(entry) => vec![self.resolve_method_entry(entry)],
        }
    }

    fn resolve_method_entry(&self, method_entry: &MethodEntry) -> AccessRule {
        let mut group_rules = Vec::new();

        for group in &method_entry.groups {
            let rule = self.resolve_entry(&AuthorityEntry::Group(group.to_string()));
            match rule {
                AccessRule::DenyAll => {
                    group_rules.push(AccessRuleNode::AnyOf(vec![]));
                }
                AccessRule::AllowAll => {
                    group_rules.push(AccessRuleNode::AllOf(vec![]));
                }
                AccessRule::Protected(node) => group_rules.push(node),
            }
        }

        AccessRule::Protected(AccessRuleNode::AnyOf(group_rules))
    }

    fn resolve_entry(&self, entry: &AuthorityEntry) -> AccessRule {
        match entry {
            AuthorityEntry::AccessRule(access_rule) => access_rule.clone(),
            AuthorityEntry::Group(name) => match self.authorities.get(name) {
                Some(entry) => {
                    // TODO: Make sure we don't have circular entries!
                    self.resolve_entry(entry)
                }
                None => AccessRule::DenyAll,
            },
            AuthorityEntry::Groups(groups) => {
                let mut group_rules = Vec::new();

                for group in groups {
                    let rule = self.resolve_entry(&AuthorityEntry::Group(group.to_string()));
                    match rule {
                        AccessRule::DenyAll => {
                            group_rules.push(AccessRuleNode::AnyOf(vec![]));
                        }
                        AccessRule::AllowAll => {
                            group_rules.push(AccessRuleNode::AllOf(vec![]));
                        }
                        AccessRule::Protected(node) => group_rules.push(node),
                    }
                }

                AccessRule::Protected(AccessRuleNode::AnyOf(group_rules))
            }
        }
    }

    pub fn get_authority_mutability(&self, key: &str) -> AccessRule {
        match self.mutability.get(key) {
            None => AccessRule::DenyAll,
            Some(entry) => self.resolve_entry(entry),
        }
    }

    pub fn set_authority_access_rule<E: Into<AuthorityEntry>>(
        &mut self,
        group_key: String,
        access_rule_entry: E,
    ) {
        self.authorities.insert(group_key, access_rule_entry.into());
    }

    pub fn set_authority_mutability<M: Into<AuthorityEntry>>(
        &mut self,
        key: String,
        method_auth: M,
    ) {
        self.mutability.insert(key, method_auth.into());
    }

    pub fn set_authority_access_rule_and_mutability<
        E: Into<AuthorityEntry>,
        M: Into<AuthorityEntry>,
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
        self.methods.insert(key.clone(), MethodEntry::group(group));
    }

    pub fn set_groups(&mut self, key: MethodKey, groups: Vec<String>) {
        self.methods
            .insert(key.clone(), MethodEntry::groups(groups));
    }

    pub fn set_main_method_group(&mut self, method: &str, group: &str) {
        let key = MethodKey::new(ObjectModuleId::Main, method);
        self.methods.insert(key.clone(), MethodEntry::group(group));
    }

    pub fn set_direct_access_group(&mut self, key: MethodKey, group: &str) {
        self.direct_methods
            .insert(key.clone(), MethodEntry::group(group));
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
