use crate::blueprints::pool::v1::constants::*;
use radix_common::types::EntityType;
use radix_common::{constants::*, ScryptoSbor};
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::locker::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::BlueprintId;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum IDAllocation {
    Object {
        blueprint_id: BlueprintId,
        global: bool,
    },
    KeyValueStore,
    GlobalAddressReservation,
    GlobalAddressPhantom {
        blueprint_id: BlueprintId,
    },
}

impl IDAllocation {
    pub fn entity_type(&self) -> EntityType {
        match self {
            IDAllocation::Object {
                blueprint_id,
                global,
            } => {
                if *global {
                    get_global_entity_type(&blueprint_id)
                } else {
                    get_internal_entity_type(&blueprint_id)
                }
            }
            IDAllocation::KeyValueStore => EntityType::InternalKeyValueStore,
            IDAllocation::GlobalAddressReservation => EntityType::InternalGenericComponent,
            IDAllocation::GlobalAddressPhantom { blueprint_id } => {
                get_global_entity_type(&blueprint_id)
            }
        }
    }
}

pub fn get_internal_entity_type(blueprint_id: &BlueprintId) -> EntityType {
    match (
        blueprint_id.package_address,
        blueprint_id.blueprint_name.as_str(),
    ) {
        (RESOURCE_PACKAGE, FUNGIBLE_VAULT_BLUEPRINT) => EntityType::InternalFungibleVault,
        (RESOURCE_PACKAGE, NON_FUNGIBLE_VAULT_BLUEPRINT) => EntityType::InternalNonFungibleVault,
        _ => EntityType::InternalGenericComponent,
    }
}

pub fn get_global_entity_type(blueprint_id: &BlueprintId) -> EntityType {
    match (
        blueprint_id.package_address,
        blueprint_id.blueprint_name.as_str(),
    ) {
        (PACKAGE_PACKAGE, PACKAGE_BLUEPRINT) => EntityType::GlobalPackage,
        (RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT) => {
            EntityType::GlobalFungibleResourceManager
        }
        (RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT) => {
            EntityType::GlobalNonFungibleResourceManager
        }
        (CONSENSUS_MANAGER_PACKAGE, CONSENSUS_MANAGER_BLUEPRINT) => {
            EntityType::GlobalConsensusManager
        }
        (CONSENSUS_MANAGER_PACKAGE, VALIDATOR_BLUEPRINT) => EntityType::GlobalValidator,
        (ACCESS_CONTROLLER_PACKAGE, ACCESS_CONTROLLER_BLUEPRINT) => {
            EntityType::GlobalAccessController
        }
        (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => EntityType::GlobalAccount,
        (IDENTITY_PACKAGE, IDENTITY_BLUEPRINT) => EntityType::GlobalIdentity,
        (POOL_PACKAGE, ONE_RESOURCE_POOL_BLUEPRINT_IDENT) => EntityType::GlobalOneResourcePool,
        (POOL_PACKAGE, TWO_RESOURCE_POOL_BLUEPRINT_IDENT) => EntityType::GlobalTwoResourcePool,
        (POOL_PACKAGE, MULTI_RESOURCE_POOL_BLUEPRINT_IDENT) => EntityType::GlobalMultiResourcePool,
        (LOCKER_PACKAGE, ACCOUNT_LOCKER_BLUEPRINT) => EntityType::GlobalAccountLocker,
        _ => EntityType::GlobalGenericComponent,
    }
}
