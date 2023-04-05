use crate::api::node_modules::metadata::*;
use crate::blueprints::package::PACKAGE_CLAIM_ROYALTY_IDENT;
use crate::blueprints::package::PACKAGE_SET_ROYALTY_CONFIG_IDENT;
use crate::blueprints::resource::*;
use crate::rule;
use crate::types::*;
use crate::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::str;
use sbor::rust::string::String;
use sbor::rust::string::ToString;

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
    pub module_id: TypedModuleId,
    pub ident: String,
}

impl MethodKey {
    pub fn new(module_id: TypedModuleId, method_ident: &str) -> Self {
        Self {
            module_id,
            ident: method_ident.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum AccessRuleEntry {
    AccessRule(AccessRule),
    Group(String),
}

impl AccessRuleEntry {
    pub fn group(name: &str) -> Self {
        Self::Group(name.to_string())
    }
}

impl From<AccessRule> for AccessRuleEntry {
    fn from(value: AccessRule) -> Self {
        AccessRuleEntry::AccessRule(value)
    }
}

impl From<String> for AccessRuleEntry {
    fn from(value: String) -> Self {
        AccessRuleEntry::Group(value)
    }
}

/// Method authorization rules for a component
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct AccessRulesConfig {
    direct_method_auth: BTreeMap<MethodKey, AccessRuleEntry>,
    method_auth: BTreeMap<MethodKey, AccessRuleEntry>,
    grouped_auth: BTreeMap<String, AccessRule>,
    default_auth: AccessRuleEntry,
    method_auth_mutability: BTreeMap<MethodKey, AccessRuleEntry>,
    grouped_auth_mutability: BTreeMap<String, AccessRule>,
    default_auth_mutability: AccessRuleEntry,
}

impl AccessRulesConfig {
    pub fn new() -> Self {
        Self {
            direct_method_auth: BTreeMap::new(),
            method_auth: BTreeMap::new(),
            grouped_auth: BTreeMap::new(),
            default_auth: AccessRuleEntry::AccessRule(AccessRule::DenyAll),
            method_auth_mutability: BTreeMap::new(),
            grouped_auth_mutability: BTreeMap::new(),
            default_auth_mutability: AccessRuleEntry::AccessRule(AccessRule::DenyAll),
        }
    }

    // TODO: Move into scrypto repo as a builder
    pub fn method<R: Into<AccessRuleEntry>>(
        mut self,
        method_name: &str,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self {
        let key = MethodKey::new(TypedModuleId::ObjectState, method_name);
        let mutability = mutability.into();

        self.method_auth
            .insert(key.clone(), AccessRuleEntry::AccessRule(method_auth));
        self.method_auth_mutability.insert(key, mutability);
        self
    }

    // TODO: Move into scrypto repo as a builder
    pub fn default<A: Into<AccessRuleEntry>, R: Into<AccessRuleEntry>>(
        mut self,
        default_auth: A,
        default_auth_mutability: R,
    ) -> Self {
        self.default_auth = default_auth.into();
        self.default_auth_mutability = default_auth_mutability.into();
        self
    }

    pub fn set_default_auth(&mut self, default_auth: AccessRuleEntry) {
        self.default_auth = default_auth;
    }

    pub fn get_access_rule(&self, is_direct_access: bool, key: &MethodKey) -> AccessRule {
        let auth = if is_direct_access {
            &self.direct_method_auth
        } else {
            &self.method_auth
        };
        match auth.get(key) {
            None => self.get_default(),
            Some(entry) => self.resolve_entry(entry),
        }
    }

    // TODO: Remove, used currently for vault access
    pub fn get_group_access_rule(&self, name: &str) -> AccessRule {
        self.grouped_auth
            .get(name)
            .cloned()
            .unwrap_or(AccessRule::DenyAll)
    }

    fn resolve_entry(&self, entry: &AccessRuleEntry) -> AccessRule {
        match entry {
            AccessRuleEntry::AccessRule(access_rule) => access_rule.clone(),
            AccessRuleEntry::Group(name) => match self.grouped_auth.get(name) {
                Some(access_rule) => access_rule.clone(),
                None => AccessRule::DenyAll,
            },
        }
    }

    pub fn get_mutability(&self, key: &MethodKey) -> AccessRule {
        self.method_auth_mutability
            .get(key)
            .cloned()
            .map(|e| self.resolve_entry(&e))
            .unwrap_or_else(|| self.get_default_mutability())
    }

    pub fn get_group_mutability(&self, key: &str) -> AccessRule {
        self.grouped_auth_mutability
            .get(key)
            .cloned()
            .unwrap_or_else(|| self.get_default_mutability())
    }

    pub fn get_default_mutability(&self) -> AccessRule {
        self.resolve_entry(&self.default_auth_mutability)
    }

    pub fn set_mutability<A: Into<AccessRuleEntry>>(&mut self, key: MethodKey, method_auth: A) {
        self.method_auth_mutability.insert(key, method_auth.into());
    }

    pub fn get_default(&self) -> AccessRule {
        self.resolve_entry(&self.default_auth)
    }

    pub fn set_method_access_rule<E: Into<AccessRuleEntry>>(
        &mut self,
        key: MethodKey,
        access_rule_entry: E,
    ) {
        self.method_auth.insert(key, access_rule_entry.into());
    }

    pub fn set_group_access_rule(&mut self, group_key: String, access_rule: AccessRule) {
        self.grouped_auth.insert(group_key, access_rule);
    }

    pub fn set_group_mutability(&mut self, key: String, method_auth: AccessRule) {
        self.grouped_auth_mutability.insert(key, method_auth);
    }

    pub fn set_group_access_rule_and_mutability(
        &mut self,
        group_key: &str,
        access_rule: AccessRule,
        mutability: AccessRule,
    ) {
        self.grouped_auth.insert(group_key.to_string(), access_rule);
        self.grouped_auth_mutability
            .insert(group_key.to_string(), mutability);
    }

    pub fn set_method_access_rule_and_mutability<
        A: Into<AccessRuleEntry>,
        M: Into<AccessRuleEntry>,
    >(
        &mut self,
        key: MethodKey,
        access_rule: A,
        mutability: M,
    ) {
        self.method_auth.insert(key.clone(), access_rule.into());
        self.method_auth_mutability.insert(key, mutability.into());
    }

    pub fn set_group_and_mutability<M: Into<AccessRuleEntry>>(
        &mut self,
        key: MethodKey,
        group: &str,
        mutability: M,
    ) {
        self.method_auth
            .insert(key.clone(), AccessRuleEntry::Group(group.to_string()));
        self.method_auth_mutability.insert(key, mutability.into());
    }

    pub fn set_direct_access_group(&mut self, key: MethodKey, group: &str) {
        self.direct_method_auth
            .insert(key.clone(), AccessRuleEntry::Group(group.to_string()));
    }

    pub fn set_method_access_rule_to_group(&mut self, key: MethodKey, group: String) {
        self.method_auth
            .insert(key.clone(), AccessRuleEntry::Group(group));
    }

    pub fn get_all_method_auth(&self) -> &BTreeMap<MethodKey, AccessRuleEntry> {
        &self.method_auth
    }

    pub fn get_all_grouped_auth(&self) -> &BTreeMap<String, AccessRule> {
        &self.grouped_auth
    }

    pub fn get_all_method_mutability(&self) -> &BTreeMap<MethodKey, AccessRuleEntry> {
        &self.method_auth_mutability
    }

    pub fn get_all_grouped_auth_mutability(&self) -> &BTreeMap<String, AccessRule> {
        &self.grouped_auth_mutability
    }
}

pub fn package_access_rules_from_owner_badge(
    owner_badge: &NonFungibleGlobalId,
) -> AccessRulesConfig {
    let mut access_rules = AccessRulesConfig::new().default(
        AccessRuleEntry::AccessRule(AccessRule::DenyAll),
        AccessRule::DenyAll,
    );
    access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(TypedModuleId::Metadata, METADATA_GET_IDENT),
        AccessRule::AllowAll,
        rule!(require(owner_badge.clone())),
    );
    access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(TypedModuleId::Metadata, METADATA_SET_IDENT),
        rule!(require(owner_badge.clone())),
        rule!(require(owner_badge.clone())),
    );
    access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(TypedModuleId::ObjectState, PACKAGE_SET_ROYALTY_CONFIG_IDENT),
        rule!(require(owner_badge.clone())),
        rule!(require(owner_badge.clone())),
    );
    access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(TypedModuleId::ObjectState, PACKAGE_CLAIM_ROYALTY_IDENT),
        rule!(require(owner_badge.clone())),
        rule!(require(owner_badge.clone())),
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
