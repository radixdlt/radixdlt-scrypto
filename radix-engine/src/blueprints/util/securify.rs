use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::{AccessRules, AccessRulesObject, AttachedAccessRules};
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::{ClientApi, ObjectModuleId};
use radix_engine_interface::blueprints::resource::*;

pub enum MethodType {
    Public,
    Group(String),
}

pub trait SecurifiedAccessRules {
    const SECURIFY_IDENT: Option<&'static str> = None;
    const OWNER_BADGE: ResourceAddress;

    fn securified_groups() -> Vec<&'static str>;

    fn other_groups() -> Vec<(&'static str, GroupEntry, AccessRule)> {
        vec![]
    }

    fn methods() -> Vec<(&'static str, MethodType)> {
        vec![]
    }

    fn set_non_owner_rules(access_rules_config: &mut AccessRulesConfig) {
        for (group, access_rule, mutability) in Self::other_groups() {
            access_rules_config.set_group_access_rule_and_mutability(
                group,
                access_rule,
                mutability,
            );
        }

        for (method, method_type) in Self::methods() {
            match method_type {
                MethodType::Public => {
                    access_rules_config.set_public(MethodKey::new(ObjectModuleId::Main, method));
                }
                MethodType::Group(group) => {
                    access_rules_config
                        .set_group(MethodKey::new(ObjectModuleId::Main, method), group.as_str());
                }
            };
        }
    }

    fn init_securified_rules<Y: ClientApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let mut access_rules = AccessRulesConfig::new();

        if let Some(securify_ident) = Self::SECURIFY_IDENT {
            access_rules.set_group(
                MethodKey::new(ObjectModuleId::Main, securify_ident),
                "securify",
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

        if let Some(securify_ident) = Self::SECURIFY_IDENT {
            access_rules_config.set_group_access_rule_and_mutability(
                "securify",
                AccessRule::DenyAll,
                AccessRule::DenyAll,
            );
            access_rules_config.set_group(
                MethodKey::new(ObjectModuleId::Main, securify_ident),
                "securify",
            );
        }

        let access_rules = AccessRules::sys_new(access_rules_config, btreemap!(), api)?;

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
        if Self::SECURIFY_IDENT.is_some() {
            access_rules.set_group_access_rule_and_mutability(
                "securify",
                AccessRule::DenyAll,
                AccessRule::DenyAll,
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

        if Self::SECURIFY_IDENT.is_some() {
            access_rules.set_group_access_rule_and_mutability(
                "securify",
                access_rule.clone(),
                this_package_rule.clone(),
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
