use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::{AccessRules, AccessRulesObject, AttachedAccessRules};
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;

fn init<Y: ClientApi<RuntimeError>>(
    owner_group_name: &str,
    public_methods: &[&str],
    api: &mut Y,
) -> Result<AccessRules, RuntimeError> {
    let mut access_rules = AccessRulesConfig::new();
    access_rules = access_rules.default(
        AccessRuleEntry::group(owner_group_name),
        AccessRuleEntry::group(owner_group_name),
    );

    for public_method in public_methods {
        access_rules.set_access_rule_and_mutability(
            MethodKey::new(NodeModuleId::SELF, public_method),
            AccessRule::AllowAll,
            AccessRule::DenyAll,
        );
    }

    let access_rules = AccessRules::sys_new(access_rules, api)?;
    Ok(access_rules)
}

fn securify_access_rules<A: AccessRulesObject, Y: ClientApi<RuntimeError>>(
    securify_ident: &str,
    owner_token_address: ResourceAddress,
    owner_group_name: &str,
    access_rules: &A,
    api: &mut Y,
) -> Result<Bucket, RuntimeError> {
    let owner_token = ResourceManager(owner_token_address);
    let (bucket, owner_local_id) = owner_token.mint_non_fungible_single_uuid((), api)?;
    access_rules.set_method_access_rule_and_mutability(
        MethodKey::new(NodeModuleId::SELF, securify_ident),
        AccessRuleEntry::AccessRule(AccessRule::DenyAll),
        AccessRule::DenyAll,
        api,
    )?;
    let global_id = NonFungibleGlobalId::new(owner_token_address, owner_local_id);
    access_rules.set_group_access_rule_and_mutability(
        owner_group_name,
        rule!(require(global_id)),
        AccessRule::DenyAll,
        api,
    )?;

    Ok(bucket)
}

pub trait SecurifiedAccessRules {
    const OWNER_GROUP_NAME: &'static str;
    const PUBLIC_METHODS: &'static [&'static str] = &[];
    const SECURIFY_IDENT: &'static str;
    const PACKAGE: PackageAddress;
    const OWNER_TOKEN: ResourceAddress;

    fn create_advanced<Y: ClientApi<RuntimeError>>(
        access_rule: AccessRule,
        mutability: AccessRule,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let access_rules = init(Self::OWNER_GROUP_NAME, Self::PUBLIC_METHODS, api)?;
        access_rules.set_method_access_rule_and_mutability(
            MethodKey::new(NodeModuleId::SELF, Self::SECURIFY_IDENT),
            AccessRuleEntry::AccessRule(AccessRule::DenyAll),
            AccessRule::DenyAll,
            api,
        )?;
        access_rules.set_group_access_rule_and_mutability(
            Self::OWNER_GROUP_NAME,
            access_rule,
            mutability,
            api,
        )?;

        Ok(access_rules)
    }

    fn create_securified<Y: ClientApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<(AccessRules, Bucket), RuntimeError> {
        let access_rules = init(Self::OWNER_GROUP_NAME, Self::PUBLIC_METHODS, api)?;
        let bucket = securify_access_rules(
            Self::SECURIFY_IDENT,
            Self::OWNER_TOKEN,
            Self::OWNER_GROUP_NAME,
            &access_rules,
            api,
        )?;
        Ok((access_rules, bucket))
    }

    fn create_presecurified<Y: ClientApi<RuntimeError>>(
        owner_id: NonFungibleGlobalId,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let access_rules = init(Self::OWNER_GROUP_NAME, Self::PUBLIC_METHODS, api)?;

        let package_id = NonFungibleGlobalId::new(
            PACKAGE_TOKEN,
            NonFungibleLocalId::bytes(scrypto_encode(&Self::PACKAGE).unwrap()).unwrap(),
        );
        let this_package_rule = rule!(require(package_id));

        let access_rule = rule!(require(owner_id));
        access_rules.set_method_access_rule_and_mutability(
            MethodKey::new(NodeModuleId::SELF, Self::SECURIFY_IDENT),
            AccessRuleEntry::AccessRule(access_rule.clone()),
            this_package_rule.clone(),
            api,
        )?;
        access_rules.set_group_access_rule_and_mutability(
            Self::OWNER_GROUP_NAME,
            access_rule,
            this_package_rule,
            api,
        )?;

        Ok(access_rules)
    }

    fn securify<Y: ClientApi<RuntimeError>>(
        receiver: RENodeId,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let access_rules = AttachedAccessRules(receiver);
        let bucket = securify_access_rules(
            Self::SECURIFY_IDENT,
            Self::OWNER_TOKEN,
            Self::OWNER_GROUP_NAME,
            &access_rules,
            api,
        )?;
        Ok(bucket)
    }
}
