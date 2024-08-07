use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use radix_engine_interface::api::{ModuleId, SystemApi};
use radix_engine_interface::blueprints::resource::*;
use radix_native_sdk::modules::role_assignment::{
    AttachedRoleAssignment, RoleAssignment, RoleAssignmentObject,
};
use radix_native_sdk::resource::ResourceManager;

pub trait SecurifiedRoleAssignment {
    const OWNER_BADGE: ResourceAddress;
    type OwnerBadgeNonFungibleData: NonFungibleData;
    const SECURIFY_ROLE: Option<&'static str> = None;

    fn create_advanced<Y: SystemApi<RuntimeError>>(
        owner_role: OwnerRole,
        api: &mut Y,
    ) -> Result<RoleAssignment, RuntimeError> {
        let mut roles = RoleAssignmentInit::new();
        if let Some(securify_role) = Self::SECURIFY_ROLE {
            roles.define_role(RoleKey::new(securify_role), AccessRule::DenyAll);
        }
        let roles = indexmap!(ModuleId::Main => roles);
        let role_assignment = RoleAssignment::create(owner_role, roles, api)?;
        Ok(role_assignment)
    }

    fn create_securified<Y: SystemApi<RuntimeError>>(
        owner_badge_data: Self::OwnerBadgeNonFungibleData,
        non_fungible_local_id: Option<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(RoleAssignment, Bucket), RuntimeError> {
        let (bucket, owner_role) =
            Self::mint_securified_badge(owner_badge_data, non_fungible_local_id, api)?;
        let mut roles = RoleAssignmentInit::new();
        if let Some(securify_role) = Self::SECURIFY_ROLE {
            roles.define_role(RoleKey::new(securify_role), AccessRule::DenyAll);
        }
        let roles = indexmap!(ModuleId::Main => roles);
        let role_assignment = RoleAssignment::create(OwnerRole::Fixed(owner_role), roles, api)?;
        Ok((role_assignment, bucket.into()))
    }

    fn mint_securified_badge<Y: SystemApi<RuntimeError>>(
        owner_badge_data: Self::OwnerBadgeNonFungibleData,
        non_fungible_local_id: Option<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(NonFungibleBucket, AccessRule), RuntimeError> {
        let owner_token = ResourceManager(Self::OWNER_BADGE);
        let (bucket, owner_local_id) = if let Some(owner_local_id) = non_fungible_local_id {
            (
                owner_token.mint_non_fungible(
                    indexmap!(
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

pub trait PresecurifiedRoleAssignment: SecurifiedRoleAssignment {
    fn create_presecurified<Y: SystemApi<RuntimeError>>(
        owner_id: NonFungibleGlobalId,
        api: &mut Y,
    ) -> Result<RoleAssignment, RuntimeError> {
        let mut roles = RoleAssignmentInit::new();
        let owner_role = rule!(require(owner_id));
        if let Some(securify_role) = Self::SECURIFY_ROLE {
            roles.define_role(RoleKey::new(securify_role), owner_role.clone());
        }

        let roles = indexmap!(
            ModuleId::Main => roles,
        );

        let role_assignment = RoleAssignment::create(
            OwnerRoleEntry {
                rule: owner_role,
                updater: OwnerRoleUpdater::Object,
            },
            roles,
            api,
        )?;
        Ok(role_assignment)
    }

    fn securify<Y: SystemApi<RuntimeError>>(
        receiver: &NodeId,
        owner_badge_data: Self::OwnerBadgeNonFungibleData,
        non_fungible_local_id: Option<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<NonFungibleBucket, RuntimeError> {
        let role_assignment = AttachedRoleAssignment(*receiver);
        if let Some(securify_role) = Self::SECURIFY_ROLE {
            role_assignment.set_role(
                ModuleId::Main,
                RoleKey::new(securify_role),
                AccessRule::DenyAll,
                api,
            )?;
        }

        let (bucket, owner_role) =
            Self::mint_securified_badge(owner_badge_data, non_fungible_local_id, api)?;

        role_assignment.set_owner_role(owner_role, api)?;

        Ok(bucket)
    }
}
