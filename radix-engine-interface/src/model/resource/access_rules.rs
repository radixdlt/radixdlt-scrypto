use radix_engine_interface::api::types::NativeFn;
use sbor::rust::collections::hash_map::Iter;
use sbor::rust::collections::{BTreeMap, HashMap};
use sbor::rust::str;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::*;

use crate::api::types::MetadataFn;
use crate::api::types::PackageFn;
use crate::model::*;
use crate::rule;
use crate::scrypto;
use crate::Describe;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[scrypto(Categorize, Encode, Decode, Describe)]
pub enum AccessRuleKey {
    ScryptoMethod(String),
    Native(NativeFn),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[scrypto(Categorize, Encode, Decode, Describe)]
pub enum AccessRuleEntry {
    AccessRule(AccessRule),
    Group(String),
}

/// Method authorization rules for a component
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(Categorize, Encode, Decode, Describe)]
pub struct AccessRules {
    method_auth: HashMap<AccessRuleKey, AccessRuleEntry>,
    grouped_auth: HashMap<String, AccessRule>,
    default_auth: AccessRule,
    method_auth_mutability: HashMap<AccessRuleKey, AccessRule>,
    grouped_auth_mutability: HashMap<String, AccessRule>,
    default_auth_mutability: AccessRule,
}

impl AccessRules {
    pub fn new() -> Self {
        Self {
            method_auth: HashMap::new(),
            grouped_auth: HashMap::new(),
            default_auth: AccessRule::DenyAll,
            method_auth_mutability: HashMap::new(),
            grouped_auth_mutability: HashMap::new(),
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
        let key = AccessRuleKey::ScryptoMethod(method_name.to_string());
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

    pub fn get_mutability(&self, key: &AccessRuleKey) -> &AccessRule {
        self.method_auth_mutability
            .get(key)
            .unwrap_or(&self.default_auth_mutability)
    }

    pub fn get_group_mutability(&self, key: &str) -> &AccessRule {
        self.grouped_auth_mutability
            .get(key)
            .unwrap_or(&self.default_auth_mutability)
    }

    pub fn set_mutability(&mut self, key: AccessRuleKey, method_auth: AccessRule) {
        self.method_auth_mutability.insert(key, method_auth);
    }

    pub fn set_group_mutability(&mut self, key: String, method_auth: AccessRule) {
        self.grouped_auth_mutability.insert(key, method_auth);
    }

    pub fn get(&self, key: &AccessRuleKey) -> &AccessRule {
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

    pub fn set_method_access_rule(&mut self, key: AccessRuleKey, access_rule: AccessRule) {
        self.method_auth
            .insert(key, AccessRuleEntry::AccessRule(access_rule));
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
        key: AccessRuleKey,
        access_rule: AccessRule,
        mutability: AccessRule,
    ) {
        self.method_auth
            .insert(key.clone(), AccessRuleEntry::AccessRule(access_rule));
        self.method_auth_mutability.insert(key, mutability);
    }

    pub fn set_group_and_mutability(
        &mut self,
        key: AccessRuleKey,
        group: String,
        mutability: AccessRule,
    ) {
        self.method_auth
            .insert(key.clone(), AccessRuleEntry::Group(group));
        self.method_auth_mutability.insert(key, mutability);
    }

    pub fn iter(&self) -> Iter<'_, AccessRuleKey, AccessRuleEntry> {
        let l = self.method_auth.iter();
        l
    }

    pub fn get_all_method_auth(&self) -> &HashMap<AccessRuleKey, AccessRuleEntry> {
        &self.method_auth
    }

    pub fn get_all_grouped_auth(&self) -> &HashMap<String, AccessRule> {
        &self.grouped_auth
    }

    pub fn get_default_auth(&self) -> &AccessRule {
        &self.default_auth
    }

    pub fn get_all_method_auth_mutability(&self) -> &HashMap<AccessRuleKey, AccessRule> {
        &self.method_auth_mutability
    }

    pub fn get_all_grouped_auth_mutability(&self) -> &HashMap<String, AccessRule> {
        &self.grouped_auth_mutability
    }

    pub fn get_default_auth_mutability(&self) -> &AccessRule {
        &self.default_auth_mutability
    }
}

pub fn package_access_rules_from_owner_badge(owner_badge: &NonFungibleAddress) -> AccessRules {
    let mut access_rules = AccessRules::new().default(AccessRule::DenyAll, AccessRule::DenyAll);
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Metadata(MetadataFn::Get)),
        AccessRule::AllowAll,
        rule!(require(owner_badge.clone())),
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Metadata(MetadataFn::Set)),
        rule!(require(owner_badge.clone())),
        rule!(require(owner_badge.clone())),
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Package(PackageFn::SetRoyaltyConfig)),
        rule!(require(owner_badge.clone())),
        rule!(require(owner_badge.clone())),
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Package(PackageFn::ClaimRoyalty)),
        rule!(require(owner_badge.clone())),
        rule!(require(owner_badge.clone())),
    );
    access_rules
}

pub fn resource_access_rules_from_owner_badge(
    owner_badge: &NonFungibleAddress,
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
