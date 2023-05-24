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

    fn create_roles<M: Into<RoleList>>(owner_rule: AccessRule, mutability: M) -> Roles {
        let mut roles = Self::role_definitions();
        roles.define_role(
            RoleKey::new(Self::OWNER_ROLE),
            owner_rule,
            mutability.into(),
        );
        roles
    }

    fn create_method_permissions(
        securify_permission: (MethodPermission, RoleList),
    ) -> BTreeMap<MethodKey, (MethodPermission, RoleList)> {
        let mut method_permissions = Self::method_permissions();
        if let Some(securify) = Self::SECURIFY_METHOD {
            method_permissions.insert(MethodKey::main(securify), securify_permission);
        }
        method_permissions
    }

    fn create_advanced<Y: ClientApi<RuntimeError>>(
        owner_rule: OwnerRule,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let (rule, mutability) = owner_rule.to_rules(Self::OWNER_ROLE);
        let roles = Self::create_roles(rule, mutability);
        let method_permissions =
            Self::create_method_permissions((MethodPermission::nobody(), RoleList::none()));
        let access_rules = AccessRules::create(method_permissions, roles, btreemap!(), api)?;

        Ok(access_rules)
    }

    fn create_securified<Y: ClientApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<(AccessRules, Bucket), RuntimeError> {
        let roles = Self::create_roles(AccessRule::DenyAll, [SELF_ROLE]);
        let method_permissions =
            Self::create_method_permissions((MethodPermission::nobody(), RoleList::none()));
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
            access_rules.update_method(
                MethodKey::main(securify),
                MethodPermission::nobody(),
                RoleList::none(),
                api,
            )?;
        }

        let global_id = NonFungibleGlobalId::new(Self::OWNER_BADGE, owner_local_id);

        access_rules.update_role(
            RoleKey::new(Self::OWNER_ROLE),
            rule!(require(global_id.clone())),
            [Self::OWNER_ROLE],
            api,
        )?;

        Ok(bucket)
    }
}

pub trait PresecurifiedAccessRules: SecurifiedAccessRules {
    fn create_presecurified<Y: ClientApi<RuntimeError>>(
        owner_id: NonFungibleGlobalId,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let roles = Self::create_roles(rule!(require(owner_id)), [SELF_ROLE]);
        let method_permissions =
            Self::create_method_permissions(([Self::OWNER_ROLE].into(), [SELF_ROLE].into()));
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
