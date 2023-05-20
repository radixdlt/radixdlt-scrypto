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
use scrypto_schema::SchemaAuthorityKey;
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
pub enum AttachedModule {
    Metadata,
    Royalty,
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum AuthorityKey {
    Main(String),
    ModuleEntryPoint(AttachedModule, String),
}

impl From<SchemaAuthorityKey> for AuthorityKey {
    fn from(value: SchemaAuthorityKey) -> Self {
        Self::Main(value.key)
    }
}

impl AuthorityKey {
    pub fn from_access_rule(rule: AuthorityRule) -> Self {
        match rule {
            AuthorityRule::Custom(key) => AuthorityKey::Main(key),
        }
    }

    pub fn main<S: Into<String>>(key: S) -> Self {
        AuthorityKey::Main(key.into())
    }

    pub fn metadata(key: &str) -> Self {
        AuthorityKey::ModuleEntryPoint(AttachedModule::Metadata, key.to_string())
    }

    pub fn royalty(key: &str) -> Self {
        AuthorityKey::ModuleEntryPoint(AttachedModule::Royalty, key.to_string())
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
            .set_main_authority_rule(
                "owner",
                rule!(require(owner_badge.clone())),
                rule!(require("owner"))
            );
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

    pub fn set_fixed_authority_rule<S: Into<String>, R: Into<AccessRule>>(
        &mut self,
        authority: S,
        rule: R,
    ) {
        self.rules.insert(
            AuthorityKey::main(authority),
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
        self.rules
            .insert(AuthorityKey::main(name.as_str()), (rule, mutability));
    }

    pub fn set_royalty_authority(&mut self, rule: AccessRule, mutability: AccessRule) {
        self.rules
            .insert(AuthorityKey::royalty(ROYALTY_AUTHORITY), (rule, mutability));
    }

    pub fn set_owner_authority(&mut self, rule: AccessRule, mutability: AccessRule) {
        self.rules.insert(AuthorityKey::main("owner"), (rule, mutability));
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
