use crate::api::node_modules::metadata::*;
use crate::api::node_modules::royalty::*;
use crate::api::types::NodeModuleId;
use crate::blueprints::resource::*;
use crate::rule;
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
pub struct MethodKey {
    pub node_module_id: NodeModuleId,
    pub ident: String,
}

impl MethodKey {
    pub fn new(node_module_id: NodeModuleId, method_ident: String) -> Self {
        Self {
            node_module_id,
            ident: method_ident,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum AccessRuleEntry {
    AccessRule(AccessRule),
    Group(String),
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
pub struct AccessRules {
    method_auth: BTreeMap<MethodKey, AccessRuleEntry>,
    grouped_auth: BTreeMap<String, AccessRule>,
    default_auth: AccessRule,
    method_auth_mutability: BTreeMap<MethodKey, AccessRule>,
    grouped_auth_mutability: BTreeMap<String, AccessRule>,
    default_auth_mutability: AccessRule,
}

impl AccessRules {
    pub fn new() -> Self {
        Self {
            method_auth: BTreeMap::new(),
            grouped_auth: BTreeMap::new(),
            default_auth: AccessRule::DenyAll,
            method_auth_mutability: BTreeMap::new(),
            grouped_auth_mutability: BTreeMap::new(),
            default_auth_mutability: AccessRule::DenyAll,
        }
    }

    // TODO: Move into scrypto repo as a builder
    pub fn method<R: Into<AccessRule>>(
        mut self,
        method_name: &str,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self {
        let key = MethodKey::new(NodeModuleId::SELF, method_name.to_string());
        let mutability = mutability.into();

        self.method_auth
            .insert(key.clone(), AccessRuleEntry::AccessRule(method_auth));
        self.method_auth_mutability.insert(key, mutability);
        self
    }

    // TODO: Move into scrypto repo as a builder
    pub fn default<R: Into<AccessRule>>(
        mut self,
        default_auth: AccessRule,
        default_auth_mutability: R,
    ) -> Self {
        self.default_auth = default_auth;
        self.default_auth_mutability = default_auth_mutability.into();
        self
    }

    pub fn set_default_auth(&mut self, default_auth: AccessRule) {
        self.default_auth = default_auth;
    }

    pub fn set_default_auth_mutability(&mut self, default_auth_mutability: AccessRule) {
        self.default_auth_mutability = default_auth_mutability;
    }

    pub fn get_mutability(&self, key: &MethodKey) -> &AccessRule {
        self.method_auth_mutability
            .get(key)
            .unwrap_or(&self.default_auth_mutability)
    }

    pub fn get_group_mutability(&self, key: &str) -> &AccessRule {
        self.grouped_auth_mutability
            .get(key)
            .unwrap_or(&self.default_auth_mutability)
    }

    pub fn set_mutability(&mut self, key: MethodKey, method_auth: AccessRule) {
        self.method_auth_mutability.insert(key, method_auth);
    }

    pub fn set_group_mutability(&mut self, key: String, method_auth: AccessRule) {
        self.grouped_auth_mutability.insert(key, method_auth);
    }

    pub fn get(&self, key: &MethodKey) -> &AccessRule {
        match self.method_auth.get(key) {
            None => &self.default_auth,
            Some(AccessRuleEntry::AccessRule(access_rule)) => access_rule,
            Some(AccessRuleEntry::Group(group_key)) => self.get_group(group_key),
        }
    }

    pub fn get_group(&self, key: &str) -> &AccessRule {
        self.grouped_auth.get(key).unwrap_or(&self.default_auth)
    }

    pub fn get_default(&self) -> &AccessRule {
        &self.default_auth
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

    pub fn set_group_access_rule_and_mutability(
        &mut self,
        group_key: String,
        access_rule: AccessRule,
        mutability: AccessRule,
    ) {
        self.grouped_auth.insert(group_key.clone(), access_rule);
        self.grouped_auth_mutability.insert(group_key, mutability);
    }

    pub fn set_access_rule_and_mutability(
        &mut self,
        key: MethodKey,
        access_rule: AccessRule,
        mutability: AccessRule,
    ) {
        self.method_auth
            .insert(key.clone(), AccessRuleEntry::AccessRule(access_rule));
        self.method_auth_mutability.insert(key, mutability);
    }

    pub fn set_group_and_mutability(
        &mut self,
        key: MethodKey,
        group: String,
        mutability: AccessRule,
    ) {
        self.method_auth
            .insert(key.clone(), AccessRuleEntry::Group(group));
        self.method_auth_mutability.insert(key, mutability);
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

    pub fn get_default_auth(&self) -> &AccessRule {
        &self.default_auth
    }

    pub fn get_all_method_auth_mutability(&self) -> &BTreeMap<MethodKey, AccessRule> {
        &self.method_auth_mutability
    }

    pub fn get_all_grouped_auth_mutability(&self) -> &BTreeMap<String, AccessRule> {
        &self.grouped_auth_mutability
    }

    pub fn get_default_auth_mutability(&self) -> &AccessRule {
        &self.default_auth_mutability
    }
}

pub fn package_access_rules_from_owner_badge(owner_badge: &NonFungibleGlobalId) -> AccessRules {
    let mut access_rules = AccessRules::new().default(AccessRule::DenyAll, AccessRule::DenyAll);
    access_rules.set_access_rule_and_mutability(
        MethodKey::new(NodeModuleId::Metadata, METADATA_GET_IDENT.to_string()),
        AccessRule::AllowAll,
        rule!(require(owner_badge.clone())),
    );
    access_rules.set_access_rule_and_mutability(
        MethodKey::new(NodeModuleId::Metadata, METADATA_SET_IDENT.to_string()),
        rule!(require(owner_badge.clone())),
        rule!(require(owner_badge.clone())),
    );
    access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            NodeModuleId::PackageRoyalty,
            PACKAGE_ROYALTY_SET_ROYALTY_CONFIG_IDENT.to_string(),
        ),
        rule!(require(owner_badge.clone())),
        rule!(require(owner_badge.clone())),
    );
    access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            NodeModuleId::PackageRoyalty,
            PACKAGE_ROYALTY_CLAIM_ROYALTY_IDENT.to_string(),
        ),
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
