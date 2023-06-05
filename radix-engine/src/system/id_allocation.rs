use radix_engine_common::types::EntityType;
use radix_engine_common::{native_addresses::*, ScryptoSbor};
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::BlueprintId;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum IDAllocation {
    Object {
        blueprint_id: BlueprintId,
        global: bool,
    },
    KeyValueStore,
    GlobalAddressOwnership,
    GlobalAddressPhantom {
        blueprint_id: BlueprintId,
    },
}

impl IDAllocation {
    pub fn is_global(&self) -> bool {
        match self {
            IDAllocation::Object { global, .. } => *global,
            IDAllocation::KeyValueStore => false,
            IDAllocation::GlobalAddressOwnership => false,
            IDAllocation::GlobalAddressPhantom { .. } => true,
        }
    }

    pub fn entity_type(&self) -> EntityType {
        match self {
            IDAllocation::Object {
                blueprint_id,
                global,
            } => {
                // FIXME final check before Babylon release!
                if *global {
                    match (
                        blueprint_id.package_address,
                        blueprint_id.blueprint_name.as_str(),
                    ) {
                        (ACCOUNT_PACKAGE, PACKAGE_BLUEPRINT) => EntityType::GlobalPackage,
                        (RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT) => {
                            EntityType::GlobalFungibleResourceManager
                        }
                        (RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT) => {
                            EntityType::GlobalNonFungibleResourceManager
                        }
                        (CONSENSUS_MANAGER_PACKAGE, CONSENSUS_MANAGER_BLUEPRINT) => {
                            EntityType::GlobalConsensusManager
                        }
                        (CONSENSUS_MANAGER_PACKAGE, VALIDATOR_BLUEPRINT) => {
                            EntityType::GlobalValidator
                        }
                        (ACCESS_CONTROLLER_PACKAGE, ACCESS_CONTROLLER_BLUEPRINT) => {
                            EntityType::GlobalAccessController
                        }
                        (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => EntityType::GlobalAccount,
                        (IDENTITY_PACKAGE, IDENTITY_BLUEPRINT) => EntityType::GlobalIdentity,
                        (POOL_PACKAGE, SINGLE_RESOURCE_POOL_CONTRIBUTE_IDENT) => {
                            EntityType::GlobalSingleResourcePool
                        }
                        (POOL_PACKAGE, TWO_RESOURCE_POOL_CONTRIBUTE_IDENT) => {
                            EntityType::GlobalTwoResourcePool
                        }
                        _ => EntityType::GlobalGenericComponent,
                    }
                } else {
                    match (
                        blueprint_id.package_address,
                        blueprint_id.blueprint_name.as_str(),
                    ) {
                        (RESOURCE_PACKAGE, FUNGIBLE_VAULT_BLUEPRINT) => {
                            EntityType::InternalFungibleVault
                        }
                        (RESOURCE_PACKAGE, NON_FUNGIBLE_VAULT_BLUEPRINT) => {
                            EntityType::InternalNonFungibleVault
                        }
                        (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => EntityType::InternalAccount,
                        _ => EntityType::InternalGenericComponent,
                    }
                }
            }
            IDAllocation::KeyValueStore => EntityType::InternalKeyValueStore,
            IDAllocation::GlobalAddressOwnership => EntityType::InternalGenericComponent,
            IDAllocation::GlobalAddressPhantom { blueprint_id } => match (
                blueprint_id.package_address,
                blueprint_id.blueprint_name.as_str(),
            ) {
                (ACCOUNT_PACKAGE, PACKAGE_BLUEPRINT) => EntityType::GlobalPackage,
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
                (POOL_PACKAGE, SINGLE_RESOURCE_POOL_CONTRIBUTE_IDENT) => {
                    EntityType::GlobalSingleResourcePool
                }
                (POOL_PACKAGE, TWO_RESOURCE_POOL_CONTRIBUTE_IDENT) => {
                    EntityType::GlobalTwoResourcePool
                }
                _ => EntityType::GlobalGenericComponent,
            },
        }
    }
}
