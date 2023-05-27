use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::{AccessRules, AccessRulesObject, AttachedAccessRules};
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;

pub enum SecurifiedRoleEntry {
    Owner {
        mutable: RoleList,
        mutable_mutable: bool,
    },
    Normal(RoleEntry)
}

pub trait SecurifiedAccessRules {
    const OWNER_BADGE: ResourceAddress;
    const OWNER_ROLE: &'static str;
    const SECURIFY_ROLE: Option<&'static str> = None;

    fn method_permissions() -> BTreeMap<MethodKey, MethodEntry>;

    fn role_definitions() -> BTreeMap<RoleKey, SecurifiedRoleEntry>;

    fn create_roles(owner_rule: RoleEntry, presecurify: bool) -> Roles {
        let role_entries = Self::role_definitions();

        let mut roles = Roles::new();
        for (role, entry) in role_entries {
            let role_entry = match entry {
                SecurifiedRoleEntry::Owner { mutable, mutable_mutable } => {
                    RoleEntry::new(owner_rule.rule.clone(), mutable, mutable_mutable)
                }
                SecurifiedRoleEntry::Normal(entry) => entry,
            };
            roles.define_role(role, role_entry);
        }

        roles.define_role(RoleKey::new(Self::OWNER_ROLE), owner_rule.clone());
        if let Some(securify_role) = Self::SECURIFY_ROLE {
            let securify_rule = if presecurify {
                owner_rule
            } else {
                RoleEntry::disabled()
            };

            roles.define_role(RoleKey::new(securify_role), securify_rule);
        }

        roles
    }

    fn create_advanced<Y: ClientApi<RuntimeError>>(
        owner_rule: OwnerRole,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let owner_role_entry = owner_rule.to_role_entry(Self::OWNER_ROLE);
        let roles = Self::create_roles(owner_role_entry, false);
        let method_permissions = Self::method_permissions();
        let access_rules = AccessRules::create(method_permissions, roles, btreemap!(), api)?;
        Ok(access_rules)
    }

    fn create_securified_badge<Y: ClientApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<(Bucket, RoleEntry), RuntimeError> {
        let owner_token = ResourceManager(Self::OWNER_BADGE);
        let (bucket, owner_local_id) = owner_token.mint_non_fungible_single_uuid((), api)?;
        let global_id = NonFungibleGlobalId::new(Self::OWNER_BADGE, owner_local_id);
        let owner_entry = RoleEntry::immutable(rule!(require(global_id)));
        Ok((bucket, owner_entry))
    }

    fn create_securified<Y: ClientApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<(AccessRules, Bucket), RuntimeError> {
        let (bucket, owner_entry) = Self::create_securified_badge(api)?;
        let roles = Self::create_roles(owner_entry, false);
        let method_permissions = Self::method_permissions();
        let access_rules = AccessRules::create(method_permissions, roles, btreemap!(), api)?;
        Ok((access_rules, bucket))
    }
}

pub trait PresecurifiedAccessRules: SecurifiedAccessRules {
    fn create_presecurified<Y: ClientApi<RuntimeError>>(
        owner_id: NonFungibleGlobalId,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let roles = Self::create_roles(
            RoleEntry::new(rule!(require(owner_id)), [SELF_ROLE], true),
                true,
        );
        let method_permissions = Self::method_permissions();
        let access_rules = AccessRules::create(method_permissions, roles, btreemap!(), api)?;
        Ok(access_rules)
    }

    fn securify<Y: ClientApi<RuntimeError>>(
        receiver: &NodeId,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let access_rules = AttachedAccessRules(*receiver);
        if let Some(securify_role) = Self::SECURIFY_ROLE {
            access_rules.update_role(
                RoleKey::new(securify_role),
                RoleEntry::disabled(),
                api,
            )?;
        }

        let (bucket, owner_entry) = Self::create_securified_badge(api)?;

        access_rules.update_role(
            RoleKey::new(Self::OWNER_ROLE),
            owner_entry,
            api,
        )?;

        Ok(bucket)
    }
}
