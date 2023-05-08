use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::{AccessRules, AccessRulesObject, AttachedAccessRules};
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::{ClientApi, ObjectModuleId};
use radix_engine_interface::blueprints::resource::*;

pub enum MethodType {
    Public,
    Custom(AccessRuleEntry, AccessRuleEntry),
}

pub trait SecurifiedAccessRules {
    const SECURIFY_IDENT: Option<&'static str> = None;
    const OWNER_BADGE: ResourceAddress;

    fn securified_groups() -> Vec<&'static str>;

    fn non_owner_methods() -> Vec<(&'static str, MethodType)> {
        vec![]
    }

    fn set_non_owner_rules(access_rules_config: &mut AccessRulesConfig) {
        for (method, method_type) in Self::non_owner_methods() {
            let (access_rule, mutability) = match method_type {
                MethodType::Public => (
                    AccessRuleEntry::AccessRule(AccessRule::AllowAll),
                    AccessRuleEntry::AccessRule(AccessRule::DenyAll),
                ),
                MethodType::Custom(access_rule, mutability) => (access_rule, mutability),
            };

            access_rules_config.set_method_access_rule_and_mutability(
                MethodKey::new(ObjectModuleId::Main, method),
                access_rule,
                mutability,
            );
        }
    }

    fn init_securified_rules<Y: ClientApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let mut access_rules = AccessRulesConfig::new();

        // TODO: Fix this up
        for securified_group in Self::securified_groups() {
            access_rules = access_rules.default(
                AccessRuleEntry::group(securified_group),
                AccessRuleEntry::group(securified_group),
            );
        }

        Self::set_non_owner_rules(&mut access_rules);
        let access_rules = AccessRules::sys_new(access_rules, btreemap!(), api)?;
        Ok(access_rules)
    }

    fn create_advanced<Y: ClientApi<RuntimeError>>(
        mut access_rules_config: AccessRulesConfig,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        Self::set_non_owner_rules(&mut access_rules_config);
        let access_rules = AccessRules::sys_new(access_rules_config, btreemap!(), api)?;

        if let Some(securify_ident) = Self::SECURIFY_IDENT {
            access_rules.set_method_access_rule_and_mutability(
                MethodKey::new(ObjectModuleId::Main, securify_ident),
                AccessRuleEntry::AccessRule(AccessRule::DenyAll),
                AccessRuleEntry::AccessRule(AccessRule::DenyAll),
                api,
            )?;
        }

        Ok(access_rules)
    }

    fn create_securified<Y: ClientApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<(AccessRules, Bucket), RuntimeError> {
        let access_rules = Self::init_securified_rules(api)?;
        let bucket = Self::securify_access_rules(&access_rules, api)?;
        Ok((access_rules, bucket))
    }

    fn securify_access_rules<A: AccessRulesObject, Y: ClientApi<RuntimeError>>(
        access_rules: &A,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let owner_token = ResourceManager(Self::OWNER_BADGE);
        let (bucket, owner_local_id) = owner_token.mint_non_fungible_single_uuid((), api)?;
        if let Some(securify_ident) = Self::SECURIFY_IDENT {
            access_rules.set_method_access_rule_and_mutability(
                MethodKey::new(ObjectModuleId::Main, securify_ident),
                AccessRuleEntry::AccessRule(AccessRule::DenyAll),
                AccessRuleEntry::AccessRule(AccessRule::DenyAll),
                api,
            )?;
        }
        let global_id = NonFungibleGlobalId::new(Self::OWNER_BADGE, owner_local_id);

        for securified_group in Self::securified_groups() {
            access_rules.set_group_access_rule_and_mutability(
                securified_group,
                rule!(require(global_id.clone())),
                rule!(require(global_id.clone())),
                api,
            )?;
        }


        Ok(bucket)
    }
}

pub trait PresecurifiedAccessRules: SecurifiedAccessRules {
    const PACKAGE: PackageAddress;

    fn create_presecurified<Y: ClientApi<RuntimeError>>(
        owner_id: NonFungibleGlobalId,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let access_rules = Self::init_securified_rules(api)?;

        let this_package_rule = rule!(require(package_of_direct_caller(Self::PACKAGE)));

        let access_rule = rule!(require(owner_id));
        if let Some(securify_ident) = Self::SECURIFY_IDENT {
            access_rules.set_method_access_rule_and_mutability(
                MethodKey::new(ObjectModuleId::Main, securify_ident),
                AccessRuleEntry::AccessRule(access_rule.clone()),
                AccessRuleEntry::AccessRule(this_package_rule.clone()),
                api,
            )?;
        }

        for securified_group in Self::securified_groups() {
            access_rules.set_group_access_rule_and_mutability(
                securified_group,
                access_rule.clone(),
                this_package_rule.clone(),
                api,
            )?;
        }


        Ok(access_rules)
    }

    fn securify<Y: ClientApi<RuntimeError>>(
        receiver: &NodeId,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let access_rules = AttachedAccessRules(*receiver);
        let bucket = Self::securify_access_rules(&access_rules, api)?;
        Ok(bucket)
    }
}
