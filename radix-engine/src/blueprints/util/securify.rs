use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::{AccessRules, AccessRulesObject, AttachedAccessRules};
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;

pub trait SecurifiedAccessRules {
    const OWNER_BADGE: ResourceAddress;
    const OWNER_ROLE: &'static str;
    const SECURIFY_METHOD: Option<&'static str> = None;

    fn method_permissions() -> BTreeMap<MethodKey, (MethodPermission, RoleList)>;

    fn role_definitions() -> Roles;

    fn create_roles(authority_rules: Roles) -> Roles {
        let mut authority_rules_to_use = Self::role_definitions();
        for (authority, (access_rule, mutability)) in authority_rules.rules {
            authority_rules_to_use.define_role(authority.key, access_rule, mutability);
        }

        authority_rules_to_use
    }

    fn create_advanced<Y: ClientApi<RuntimeError>>(
        role_definitions: Roles,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let roles = Self::create_roles(role_definitions);
        let mut method_permissions = Self::method_permissions();

        if let Some(securify) = Self::SECURIFY_METHOD {
            method_permissions.insert(
                MethodKey::main(securify),
                (MethodPermission::nobody(), RoleList::none()),
            );
        }

        let access_rules = AccessRules::create(method_permissions, roles, btreemap!(), api)?;

        Ok(access_rules)
    }

    fn create_securified<Y: ClientApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<(AccessRules, Bucket), RuntimeError> {
        let roles = Self::create_roles(Roles::new());
        let method_permissions = Self::method_permissions();
        let access_rules = AccessRules::create(method_permissions, roles, btreemap!(), api)?;
        let bucket = Self::securify_access_rules(&access_rules, api)?;
        Ok((access_rules, bucket))
    }

    fn securify_access_rules<A: AccessRulesObject, Y: ClientApi<RuntimeError>>(
        access_rules: &A,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let owner_token = ResourceManager(Self::OWNER_BADGE);
        let (bucket, owner_local_id) = owner_token.mint_non_fungible_single_uuid((), api)?;
        if let Some(securify) = Self::SECURIFY_METHOD {
            access_rules.set_method_permission_and_mutability(
                MethodKey::main(securify),
                MethodPermission::nobody(),
                RoleList::none(),
                api,
            )?;
        }

        let global_id = NonFungibleGlobalId::new(Self::OWNER_BADGE, owner_local_id);
        access_rules.define_role(
            RoleKey::new(Self::OWNER_ROLE),
            rule!(require(global_id.clone())),
            api,
        )?;
        access_rules.set_role_mutability(Self::OWNER_ROLE, [Self::OWNER_ROLE], api)?;

        Ok(bucket)
    }
}

pub trait PresecurifiedAccessRules: SecurifiedAccessRules {
    fn create_presecurified<Y: ClientApi<RuntimeError>>(
        owner_id: NonFungibleGlobalId,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let mut roles = Self::create_roles(Roles::new());
        roles.define_role(Self::OWNER_ROLE, rule!(require(owner_id)), [SELF_ROLE]);

        let mut method_permissions = Self::method_permissions();
        if let Some(securify) = Self::SECURIFY_METHOD {
            method_permissions.insert(
                MethodKey::main(securify),
                ([Self::OWNER_ROLE].into(), [SELF_ROLE].into()),
            );
        }

        let access_rules = AccessRules::create(method_permissions, roles, btreemap!(), api)?;
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
