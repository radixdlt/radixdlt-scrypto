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

    fn method_permissions() -> BTreeMap<MethodKey, MethodEntry>;

    fn role_definitions() -> Roles;

    fn create_roles(owner_rule: RoleEntry) -> Roles {
        let mut roles = Self::role_definitions();
        roles.define_role(RoleKey::new(Self::OWNER_ROLE), owner_rule);
        roles
    }

    fn create_method_permissions(
        securify_permission: MethodEntry,
    ) -> BTreeMap<MethodKey, MethodEntry> {
        let mut method_permissions = Self::method_permissions();
        if let Some(securify) = Self::SECURIFY_METHOD {
            method_permissions.insert(MethodKey::main(securify), securify_permission);
        }
        method_permissions
    }

    fn create_advanced<Y: ClientApi<RuntimeError>>(
        owner_rule: OwnerRole,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let owner_role_entry = owner_rule.to_role_entry(Self::OWNER_ROLE);
        let roles = Self::create_roles(owner_role_entry);
        let method_permissions = Self::create_method_permissions(MethodEntry::disabled());
        let access_rules = AccessRules::create(method_permissions, roles, btreemap!(), api)?;

        Ok(access_rules)
    }

    fn create_securified<Y: ClientApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<(AccessRules, Bucket), RuntimeError> {
        let roles = Self::create_roles(RoleEntry::new(AccessRule::DenyAll, [SELF_ROLE], true));
        let method_permissions = Self::create_method_permissions(MethodEntry::disabled());
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
            false,
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
        let roles = Self::create_roles(RoleEntry::new(rule!(require(owner_id)), [SELF_ROLE], true));
        let method_permissions =
            Self::create_method_permissions(MethodEntry::new([Self::OWNER_ROLE], [SELF_ROLE]));
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
