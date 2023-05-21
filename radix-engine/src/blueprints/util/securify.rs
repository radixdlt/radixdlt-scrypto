use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::{AccessRules, AccessRulesObject, AttachedAccessRules};
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;

pub trait SecurifiedAccessRules {
    const OWNER_BADGE: ResourceAddress;
    const SECURIFY_AUTHORITY: Option<&'static str> = None;

    fn protected_module_methods() -> BTreeMap<MethodKey, Vec<String>>;

    fn authority_rules() -> Roles;

    fn create_config(authority_rules: Roles) -> Roles {
        let mut authority_rules_to_use = Self::authority_rules();
        for (authority, (access_rule, mutability)) in authority_rules.rules {
            authority_rules_to_use.define_role(authority.key, access_rule, mutability);
        }

        authority_rules_to_use
    }

    fn init_securified_rules<Y: ClientApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let authority_rules = Self::create_config(Roles::new());
        let protected_module_methods = Self::protected_module_methods();
        let access_rules =
            AccessRules::create(protected_module_methods, authority_rules, btreemap!(), api)?;
        Ok(access_rules)
    }

    fn create_advanced<Y: ClientApi<RuntimeError>>(
        authority_rules: Roles,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let mut authority_rules = Self::create_config(authority_rules);

        if let Some(securify) = Self::SECURIFY_AUTHORITY {
            authority_rules.define_role(securify, AccessRule::DenyAll, AccessRule::DenyAll);
        }

        let protected_module_methods = Self::protected_module_methods();
        let access_rules =
            AccessRules::create(protected_module_methods, authority_rules, btreemap!(), api)?;

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
        if let Some(securify) = Self::SECURIFY_AUTHORITY {
            access_rules.set_authority_rule_and_mutability(
                AuthorityKey::new(securify),
                AccessRule::DenyAll,
                AccessRule::DenyAll,
                api,
            )?;
        }
        let global_id = NonFungibleGlobalId::new(Self::OWNER_BADGE, owner_local_id);

        access_rules.set_authority_rule_and_mutability(
            AuthorityKey::new("owner"),
            rule!(require(global_id.clone())),
            rule!(require("owner")),
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

        if let Some(securify) = Self::SECURIFY_AUTHORITY {
            access_rules.set_authority_rule_and_mutability(
                AuthorityKey::new(securify),
                access_rule.clone(),
                this_package_rule.clone(),
                api,
            )?;
        }

        access_rules.set_authority_rule_and_mutability(
            AuthorityKey::new("owner"),
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
