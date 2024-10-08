use crate::blueprints::access_controller::*;
use crate::blueprints::account::*;
use crate::blueprints::identity::*;
use crate::blueprints::package::*;
use crate::blueprints::pool::*;
use crate::internal_prelude::*;
use lazy_static::lazy_static;

/// This function resolves the [`BlueprintId`] based on the entity type for native blueprints.
///
/// If the entity type is [`EntityType::GlobalGenericComponent`] then this function returns [`None`]
/// since it can't tell what the [`BlueprintId`] of a generic component is. Otherwise, in the case
/// where the [`EntityType`] belongs to a native blueprint this function will translate it into the
/// appropriate [`BlueprintId`].
pub fn resolve_main_module_blueprint_id(entity_type: EntityType) -> Option<&'static BlueprintId> {
    match entity_type {
        EntityType::GlobalPackage => Some(&PACKAGE_BLUEPRINT_ID),
        EntityType::GlobalConsensusManager => Some(&CONSENSUS_MANAGER_BLUEPRINT_ID),
        EntityType::GlobalValidator => Some(&VALIDATOR_BLUEPRINT_ID),
        EntityType::GlobalTransactionTracker => Some(&TRANSACTION_TRACKER_BLUEPRINT_ID),
        EntityType::GlobalAccessController => Some(&ACCESS_CONTROLLER_BLUEPRINT_ID),
        EntityType::GlobalOneResourcePool => Some(&ONE_RESOURCE_POOL_BLUEPRINT_ID),
        EntityType::GlobalTwoResourcePool => Some(&TWO_RESOURCE_POOL_BLUEPRINT_ID),
        EntityType::GlobalMultiResourcePool => Some(&MULTI_RESOURCE_POOL_BLUEPRINT_ID),
        EntityType::GlobalAccountLocker => Some(&ACCOUNT_LOCKER_BLUEPRINT_ID),
        EntityType::GlobalFungibleResourceManager => Some(&FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT_ID),
        EntityType::GlobalNonFungibleResourceManager => {
            Some(&NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT_ID)
        }
        EntityType::InternalFungibleVault => Some(&FUNGIBLE_VAULT_BLUEPRINT_ID),
        EntityType::InternalNonFungibleVault => Some(&NON_FUNGIBLE_VAULT_BLUEPRINT_ID),
        EntityType::GlobalAccount
        | EntityType::GlobalPreallocatedSecp256k1Account
        | EntityType::GlobalPreallocatedEd25519Account => Some(&ACCOUNT_BLUEPRINT_ID),
        EntityType::GlobalIdentity
        | EntityType::GlobalPreallocatedSecp256k1Identity
        | EntityType::GlobalPreallocatedEd25519Identity => Some(&IDENTITY_BLUEPRINT_ID),
        EntityType::InternalGenericComponent
        | EntityType::InternalKeyValueStore
        | EntityType::GlobalGenericComponent => None,
    }
}

/// This function resolves the [`BlueprintId`] of the invoked blueprint given the [`BlueprintId`] of
/// the main module and the [`ModuleId`] being invoked.
pub fn resolve_invoked_blueprint_id(
    main_module_blueprint_id: &BlueprintId,
    module_id: ModuleId,
) -> Option<&BlueprintId> {
    match module_id {
        ModuleId::Main => Some(main_module_blueprint_id),
        // TODO: We could improve this to take into account if the blueprint has these modules or
        // not. For the time being this doesn't seem to be needed.
        ModuleId::Metadata => Some(&METADATA_BLUEPRINT_ID),
        ModuleId::Royalty => Some(&ROYALTY_BLUEPRINT_ID),
        ModuleId::RoleAssignment => Some(&ROLE_ASSIGNMENT_BLUEPRINT_ID),
    }
}

macro_rules! define_static_blueprint_id {
    (
        $(
            $name: ident => ($package_address: expr, $blueprint_name: expr)
        ),* $(,)?
    ) => {
        paste::paste! {
            lazy_static! {
                $(
                    static ref [< $name:upper _BLUEPRINT_ID >]: BlueprintId = BlueprintId {
                        package_address: $package_address,
                        blueprint_name: $blueprint_name.to_owned(),
                    };
                )*
            }
        }
    };
}

define_static_blueprint_id! {
    package => (PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
    consensus_manager => (CONSENSUS_MANAGER_PACKAGE, CONSENSUS_MANAGER_BLUEPRINT),
    validator => (CONSENSUS_MANAGER_PACKAGE, VALIDATOR_BLUEPRINT),
    transaction_tracker => (TRANSACTION_TRACKER_PACKAGE, TRANSACTION_TRACKER_BLUEPRINT),
    account => (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT),
    identity => (IDENTITY_PACKAGE, IDENTITY_BLUEPRINT),
    access_controller => (ACCESS_CONTROLLER_PACKAGE, ACCESS_CONTROLLER_BLUEPRINT),
    one_resource_pool => (POOL_PACKAGE, ONE_RESOURCE_POOL_BLUEPRINT),
    two_resource_pool => (POOL_PACKAGE, TWO_RESOURCE_POOL_BLUEPRINT),
    multi_resource_pool => (POOL_PACKAGE, MULTI_RESOURCE_POOL_BLUEPRINT),
    account_locker => (LOCKER_PACKAGE, ACCOUNT_LOCKER_BLUEPRINT),
    fungible_resource_manager => (RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
    non_fungible_resource_manager => (RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
    fungible_vault => (RESOURCE_PACKAGE, FUNGIBLE_VAULT_BLUEPRINT),
    non_fungible_vault => (RESOURCE_PACKAGE, NON_FUNGIBLE_VAULT_BLUEPRINT),
    metadata => (METADATA_MODULE_PACKAGE, METADATA_BLUEPRINT),
    role_assignment => (ROLE_ASSIGNMENT_MODULE_PACKAGE, ROLE_ASSIGNMENT_BLUEPRINT),
    royalty => (ROYALTY_MODULE_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT),
}
