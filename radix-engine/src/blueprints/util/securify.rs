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

    fn authorities() -> Vec<(&'static str, AccessRule, AccessRule)> {
        vec![]
    }

    fn methods() -> Vec<(&'static str, MethodType)> {
        vec![]
    }

    fn create_config(authority_rules: AuthorityRules) -> AccessRulesConfig {
        let mut config = AccessRulesConfig::new();

        for (method, method_type) in Self::methods() {
            let method_key = MethodKey::new(ObjectModuleId::Main, method);
            match method_type {
                MethodType::Public => {
                    config.set_public(method_key);
                }
                MethodType::Group(group) => {
                    config.set_group(method_key, group.as_str());
                }
            };
        }

        if let Some(securify_ident) = Self::SECURIFY_IDENT {
            config.set_group(
                MethodKey::new(ObjectModuleId::Main, securify_ident),
                "securify",
            );
        }

        for (authority, access_rule, mutability) in Self::authorities() {
            config.set_authority(
                authority,
                access_rule,
                mutability,
            );
        }

        for (authority, (access_rule, mutability)) in authority_rules.rules {
            config.set_authority(
                authority.as_str(),
                access_rule,
                mutability,
            );
        }

        config
    }

    fn init_securified_rules<Y: ClientApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let config = Self::create_config(AuthorityRules::new());
        let access_rules = AccessRules::sys_new(config, btreemap!(), api)?;
        Ok(access_rules)
    }

    fn create_advanced<Y: ClientApi<RuntimeError>>(
        authority_rules: AuthorityRules,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let mut config = Self::create_config(authority_rules);

        if let Some(securify_ident) = Self::SECURIFY_IDENT {
            config.set_authority(
                "securify",
                AccessRule::DenyAll,
                AccessRule::DenyAll,
            );
        }

        let access_rules = AccessRules::sys_new(config, btreemap!(), api)?;

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

        access_rules.set_group_access_rule_and_mutability(
            "owner",
            rule!(require(global_id.clone())),
            rule!(require(global_id.clone())),
            api,
        )?;

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

        access_rules.set_group_access_rule_and_mutability(
            "owner",
            access_rule.clone(),
            this_package_rule.clone(),
            api,
        )?;

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
