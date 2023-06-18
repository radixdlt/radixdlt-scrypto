use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::{AccessRules, AccessRulesObject, AttachedAccessRules};
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::node_modules::metadata::METADATA_SETTER_ROLE;
use radix_engine_interface::api::{ClientApi, ObjectModuleId};
use radix_engine_interface::blueprints::resource::*;

pub enum SecurifiedRoleEntry {
    Owner { updaters: RoleList },
    Normal(AccessRule),
}

pub trait SecurifiedAccessRules {
    const OWNER_BADGE: ResourceAddress;
    const SECURIFY_ROLE: Option<&'static str> = None;

    fn role_definitions() -> BTreeMap<RoleKey, SecurifiedRoleEntry>;

    fn create_roles(owner_rule: AccessRule, presecurify: bool) -> BTreeMap<ObjectModuleId, Roles> {
        let role_entries = Self::role_definitions();

        let mut roles = Roles::new();
        for (role, entry) in role_entries {
            match entry {
                SecurifiedRoleEntry::Owner { updaters } => {
                    if !updaters.list.is_empty() {
                        let role_entry = RoleEntry::new(owner_rule.clone(), updaters);
                        roles.define_mutable_role(role, role_entry);
                    } else {
                        roles.define_immutable_role(role, owner_rule.clone())
                    }
                }
                SecurifiedRoleEntry::Normal(rule) => {
                    roles.define_immutable_role(role, rule.clone())
                }
            };
        }

        if presecurify {
            let entry = RoleEntry::new(owner_rule.clone(), [SELF_ROLE]);
            roles.define_mutable_role(RoleKey::new(OWNER_ROLE), entry);
        } else {
            roles.define_immutable_role(RoleKey::new(OWNER_ROLE), owner_rule.clone());
        }

        if let Some(securify_role) = Self::SECURIFY_ROLE {
            if presecurify {
                roles.define_mutable_role(
                    RoleKey::new(securify_role),
                    RoleEntry::new(owner_rule.clone(), [SELF_ROLE]),
                );
            } else {
                roles.define_immutable_role(RoleKey::new(securify_role), AccessRule::DenyAll);
            };
        }

        let mut metadata_roles = Roles::new();
        metadata_roles.define_immutable_role(METADATA_SETTER_ROLE, owner_rule);

        btreemap!(
            ObjectModuleId::Main => roles,
            ObjectModuleId::Metadata => metadata_roles,
        )
    }

    fn create_advanced<Y: ClientApi<RuntimeError>>(
        owner_rule: OwnerRole,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        // FIXME: Remove to_role_entry mapping
        let owner_rule = owner_rule.to_role_entry(OWNER_ROLE).rule;
        let roles = Self::create_roles(owner_rule, false);
        let access_rules = AccessRules::create(OwnerRole::None, roles, api)?;
        Ok(access_rules)
    }

    fn create_securified_badge<Y: ClientApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<(Bucket, AccessRule), RuntimeError> {
        let owner_token = ResourceManager(Self::OWNER_BADGE);
        let (bucket, owner_local_id) = owner_token.mint_non_fungible_single_uuid((), api)?;
        let global_id = NonFungibleGlobalId::new(Self::OWNER_BADGE, owner_local_id);
        Ok((bucket, rule!(require(global_id))))
    }

    fn create_securified<Y: ClientApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<(AccessRules, Bucket), RuntimeError> {
        let (bucket, owner_rule) = Self::create_securified_badge(api)?;
        let roles = Self::create_roles(owner_rule, false);
        let access_rules = AccessRules::create(OwnerRole::None, roles, api)?;
        Ok((access_rules, bucket))
    }
}

pub trait PresecurifiedAccessRules: SecurifiedAccessRules {
    fn create_presecurified<Y: ClientApi<RuntimeError>>(
        owner_id: NonFungibleGlobalId,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let roles = Self::create_roles(rule!(require(owner_id)), true);

        let access_rules = AccessRules::create(OwnerRole::None, roles, api)?;
        Ok(access_rules)
    }

    fn securify<Y: ClientApi<RuntimeError>>(
        receiver: &NodeId,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let access_rules = AttachedAccessRules(*receiver);
        if let Some(securify_role) = Self::SECURIFY_ROLE {
            access_rules.update_role(
                ObjectModuleId::Main,
                RoleKey::new(securify_role),
                Some(AccessRule::DenyAll),
                true,
                api,
            )?;
        }

        let (bucket, owner_rule) = Self::create_securified_badge(api)?;

        access_rules.update_role(
            ObjectModuleId::Main,
            RoleKey::new(OWNER_ROLE),
            Some(owner_rule),
            true,
            api,
        )?;

        Ok(bucket)
    }
}
