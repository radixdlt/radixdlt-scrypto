use crate::blueprints::access_controller::*;
use crate::blueprints::account::*;
use crate::blueprints::identity::*;
use crate::blueprints::package::*;
use crate::blueprints::pool::*;
use crate::internal_prelude::*;

/// This function resolves the [`BlueprintId`] based on the entity type for native blueprints.
///
/// If the entity type is [`EntityType::GlobalGenericComponent`] then this function returns [`None`]
/// since it can't tell what the [`BlueprintId`] of a generic component is. Otherwise, in the case
/// where the [`EntityType`] belongs to a native blueprint this function will translate it into the
/// appropriate [`BlueprintId`].
pub fn resolve_main_module_blueprint_id(entity_type: EntityType) -> Option<&'static BlueprintId> {
    match entity_type {
        EntityType::GlobalPackage => Some(package_blueprint_id()),
        EntityType::GlobalConsensusManager => Some(consensus_manager_blueprint_id()),
        EntityType::GlobalValidator => Some(validator_blueprint_id()),
        EntityType::GlobalTransactionTracker => Some(transaction_tracker_blueprint_id()),
        EntityType::GlobalAccessController => Some(access_controller_blueprint_id()),
        EntityType::GlobalOneResourcePool => Some(one_resource_pool_blueprint_id()),
        EntityType::GlobalTwoResourcePool => Some(two_resource_pool_blueprint_id()),
        EntityType::GlobalMultiResourcePool => Some(multi_resource_pool_blueprint_id()),
        EntityType::GlobalAccountLocker => Some(account_locker_blueprint_id()),
        EntityType::GlobalFungibleResourceManager => Some(fungible_resource_manager_blueprint_id()),
        EntityType::GlobalNonFungibleResourceManager => {
            Some(non_fungible_resource_manager_blueprint_id())
        }
        EntityType::InternalFungibleVault => Some(fungible_vault_blueprint_id()),
        EntityType::InternalNonFungibleVault => Some(non_fungible_vault_blueprint_id()),
        EntityType::GlobalAccount
        | EntityType::GlobalPreallocatedSecp256k1Account
        | EntityType::GlobalPreallocatedEd25519Account => Some(account_blueprint_id()),
        EntityType::GlobalIdentity
        | EntityType::GlobalPreallocatedSecp256k1Identity
        | EntityType::GlobalPreallocatedEd25519Identity => Some(identity_blueprint_id()),
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
        ModuleId::Metadata => Some(metadata_blueprint_id()),
        ModuleId::Royalty => Some(royalty_blueprint_id()),
        ModuleId::RoleAssignment => Some(role_assignment_blueprint_id()),
    }
}

macro_rules! define_static_blueprint_id {
    (
        $(
            $name: ident => ($package_address: expr, $blueprint_name: expr)
        ),* $(,)?
    ) => {
        paste::paste! {
            $(
                pub fn [< $name _blueprint_id >]() -> &'static BlueprintId {
                    static BLUEPRINT_ID: std::sync::OnceLock<crate::prelude::BlueprintId> =
                        std::sync::OnceLock::new();
                    BLUEPRINT_ID.get_or_init(|| BlueprintId {
                        package_address: $package_address,
                        blueprint_name: $blueprint_name.to_owned(),
                    })
                }
            )*
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
