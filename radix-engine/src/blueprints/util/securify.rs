use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRulesObject;
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;

pub trait SecurifiedAccessRules {
    const OWNER_GROUP_NAME: &'static str;
    const SECURIFY_IDENT: &'static str;
    const PACKAGE: PackageAddress;
    const OWNER_TOKEN: ResourceAddress;

    fn presecurified<A: AccessRulesObject, Y: ClientApi<RuntimeError>>(
        owner_id: NonFungibleGlobalId,
        access_rules: &A,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let package_id = NonFungibleGlobalId::new(
            PACKAGE_TOKEN,
            NonFungibleLocalId::bytes(scrypto_encode(&Self::PACKAGE).unwrap()).unwrap(),
        );
        let this_package_rule = rule!(require(package_id));

        let access_rule = rule!(require(owner_id));
        access_rules.set_method_access_rule_and_mutability(
            MethodKey::new(
                NodeModuleId::SELF,
                Self::SECURIFY_IDENT,
            ),
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

        Ok(())
    }

    fn advanced<A: AccessRulesObject, Y: ClientApi<RuntimeError>>(
        access_rule: AccessRule,
        mutability: AccessRule,
        access_rules: &A,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        access_rules.set_method_access_rule_and_mutability(
            MethodKey::new(
                NodeModuleId::SELF,
                Self::SECURIFY_IDENT,
            ),
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

        Ok(())
    }

    fn securify<A: AccessRulesObject, Y: ClientApi<RuntimeError>>(access_rules: &A, api: &mut Y) -> Result<Bucket, RuntimeError> {
        let owner_token = ResourceManager(Self::OWNER_TOKEN);
        let (bucket, owner_local_id) = owner_token.mint_non_fungible_single_uuid((), api)?;
        access_rules.set_method_access_rule_and_mutability(
            MethodKey::new(
                NodeModuleId::SELF,
                Self::SECURIFY_IDENT,
            ),
            AccessRuleEntry::AccessRule(AccessRule::DenyAll),
            AccessRule::DenyAll,
            api,
        )?;
        let global_id = NonFungibleGlobalId::new(Self::OWNER_TOKEN, owner_local_id);
        access_rules.set_group_access_rule_and_mutability(
            Self::OWNER_GROUP_NAME,
            rule!(require(global_id)),
            AccessRule::DenyAll,
            api,
        )?;

        Ok(bucket)
    }
}

