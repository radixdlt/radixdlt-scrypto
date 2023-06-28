use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::{AccessRules, AccessRulesObject, AttachedAccessRules};
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::{ClientApi, ObjectModuleId};
use radix_engine_interface::blueprints::resource::*;

pub trait SecurifiedAccessRules {
    const OWNER_BADGE: ResourceAddress;
    type OwnerBadgeNonFungibleData: NonFungibleData;
    const SECURIFY_ROLE: Option<&'static str> = None;

    fn create_advanced<Y: ClientApi<RuntimeError>>(
        owner_role: OwnerRole,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let mut roles = RolesInit::new();
        if let Some(securify_role) = Self::SECURIFY_ROLE {
            roles.define_immutable_role(RoleKey::new(securify_role), AccessRule::DenyAll);
        }
        let roles = btreemap!(ObjectModuleId::Main => roles);
        let access_rules = AccessRules::create(owner_role, roles, api)?;
        Ok(access_rules)
    }

    fn create_securified<Y: ClientApi<RuntimeError>>(
        owner_badge_data: Self::OwnerBadgeNonFungibleData,
        non_fungible_local_id: Option<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(AccessRules, Bucket), RuntimeError> {
        let (bucket, owner_rule) =
            Self::mint_securified_badge(owner_badge_data, non_fungible_local_id, api)?;
        let mut roles = RolesInit::new();
        if let Some(securify_role) = Self::SECURIFY_ROLE {
            roles.define_immutable_role(RoleKey::new(securify_role), AccessRule::DenyAll);
        }
        let roles = btreemap!(ObjectModuleId::Main => roles);
        let access_rules = AccessRules::create(OwnerRole::Fixed(owner_rule), roles, api)?;
        Ok((access_rules, bucket))
    }

    fn mint_securified_badge<Y: ClientApi<RuntimeError>>(
        owner_badge_data: Self::OwnerBadgeNonFungibleData,
        non_fungible_local_id: Option<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(Bucket, AccessRule), RuntimeError> {
        let owner_token = ResourceManager(Self::OWNER_BADGE);
        let (bucket, owner_local_id) = if let Some(owner_local_id) = non_fungible_local_id {
            (
                owner_token.mint_non_fungible(
                    btreemap!(
                        owner_local_id.clone() => owner_badge_data
                    ),
                    api,
                )?,
                owner_local_id,
            )
        } else {
            owner_token.mint_non_fungible_single_ruid(owner_badge_data, api)?
        };
        let global_id = NonFungibleGlobalId::new(Self::OWNER_BADGE, owner_local_id);
        Ok((bucket, rule!(require(global_id))))
    }
}

pub trait PresecurifiedAccessRules: SecurifiedAccessRules {
    fn create_presecurified<Y: ClientApi<RuntimeError>>(
        owner_id: NonFungibleGlobalId,
        api: &mut Y,
    ) -> Result<AccessRules, RuntimeError> {
        let mut roles = RolesInit::new();
        let owner_rule = rule!(require(owner_id));
        if let Some(securify_role) = Self::SECURIFY_ROLE {
            roles.define_mutable_role(RoleKey::new(securify_role), owner_rule.clone());
        }

        let roles = btreemap!(
            ObjectModuleId::Main => roles,
        );

        let access_rules =
            AccessRules::create(OwnerRole::UpdatableByObject(owner_rule), roles, api)?;
        Ok(access_rules)
    }

    fn securify<Y: ClientApi<RuntimeError>>(
        receiver: &NodeId,
        owner_badge_data: Self::OwnerBadgeNonFungibleData,
        non_fungible_local_id: Option<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let access_rules = AttachedAccessRules(*receiver);
        if let Some(securify_role) = Self::SECURIFY_ROLE {
            access_rules.set_and_lock_role(
                ObjectModuleId::Main,
                RoleKey::new(securify_role),
                AccessRule::DenyAll,
                api,
            )?;
        }

        let (bucket, owner_rule) =
            Self::mint_securified_badge(owner_badge_data, non_fungible_local_id, api)?;

        access_rules.set_and_lock_owner_role(owner_rule, api)?;

        Ok(bucket)
    }
}
