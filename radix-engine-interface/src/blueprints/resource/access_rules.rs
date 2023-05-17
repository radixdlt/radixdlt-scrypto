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
use utils::btreemap;

use super::AccessRule;

pub const METADATA_AUTHORITY: &str = "metadata";
pub const ROYALTY_AUTHORITY: &str = "royalty";

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
    pub fn new(module_id: ObjectModuleId, method_ident: &str) -> Self {
        Self {
            module_id,
            ident: method_ident.to_string(),
        }
    }
}

impl From<String> for AccessRule {
    fn from(name: String) -> Self {
        AccessRule::Protected(AccessRuleNode::Authority(AuthorityRule::Custom(name)))
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum AuthorityKey {
    Owner,
    Module(ObjectModuleId, String),
}

impl AuthorityKey {
    pub fn from_access_rule(module_id: ObjectModuleId, rule: AuthorityRule) -> Self {
        match rule {
            AuthorityRule::Owner => AuthorityKey::Owner,
            AuthorityRule::Custom(key) => AuthorityKey::Module(module_id, key),
        }
    }

    pub fn module(module_id: ObjectModuleId, key: &str) -> Self {
        AuthorityKey::Module(module_id, key.to_string())
    }

    pub fn main(key: &str) -> Self {
        AuthorityKey::Module(ObjectModuleId::Main, key.to_string())
    }

    pub fn metadata(key: &str) -> Self {
        AuthorityKey::Module(ObjectModuleId::Metadata, key.to_string())
    }

    pub fn royalty(key: &str) -> Self {
        AuthorityKey::Module(ObjectModuleId::Royalty, key.to_string())
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct AuthorityRules {
    pub rules: BTreeMap<AuthorityKey, (AccessRule, AccessRule)>,
}

impl AuthorityRules {
    pub fn new() -> Self {
        Self { rules: btreemap!() }
    }

    pub fn new_with_owner_authority(owner_badge: &NonFungibleGlobalId) -> AuthorityRules {
        let mut authority_rules = AuthorityRules::new();
        authority_rules
            .set_owner_authority(rule!(require(owner_badge.clone())), rule!(require_owner()));
        authority_rules
    }

    pub fn set_rule(
        &mut self,
        authority_key: AuthorityKey,
        rule: AccessRule,
        mutability: AccessRule,
    ) {
        self.rules.insert(authority_key, (rule, mutability));
    }

    pub fn redirect_to_fixed<S: Into<String>>(&mut self, authority: S, redirect_to: &str) {
        let name = authority.into();
        self.rules.insert(
            AuthorityKey::module(ObjectModuleId::Main, name.as_str()),
            (rule!(require(redirect_to)), AccessRule::DenyAll),
        );
    }

    pub fn set_fixed_main_authority_rule<S: Into<String>, R: Into<AccessRule>>(
        &mut self,
        authority: S,
        rule: R,
    ) {
        let name = authority.into();
        self.rules.insert(
            AuthorityKey::module(ObjectModuleId::Main, name.as_str()),
            (rule.into(), AccessRule::DenyAll),
        );
    }

    pub fn set_main_authority_rule<S: Into<String>>(
        &mut self,
        authority: S,
        rule: AccessRule,
        mutability: AccessRule,
    ) {
        let name = authority.into();
        self.rules.insert(
            AuthorityKey::module(ObjectModuleId::Main, name.as_str()),
            (rule, mutability),
        );
    }

    pub fn set_metadata_authority(&mut self, rule: AccessRule, mutability: AccessRule) {
        self.rules.insert(
            AuthorityKey::module(ObjectModuleId::Metadata, METADATA_AUTHORITY),
            (rule, mutability),
        );
    }

    pub fn set_royalty_authority(&mut self, rule: AccessRule, mutability: AccessRule) {
        self.rules.insert(
            AuthorityKey::module(ObjectModuleId::Royalty, ROYALTY_AUTHORITY),
            (rule, mutability),
        );
    }

    pub fn set_owner_authority(&mut self, rule: AccessRule, mutability: AccessRule) {
        self.rules.insert(AuthorityKey::Owner, (rule, mutability));
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
