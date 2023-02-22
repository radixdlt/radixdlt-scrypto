use super::node::{RENodeInit, RENodeModuleInit};
use crate::errors::{KernelError, RuntimeError};
use crate::kernel::actor::{ExecutionMode, ResolvedActor, ResolvedReceiver};
use crate::kernel::kernel_api::LockFlags;
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{
    AccessControllerOffset, AccountOffset, AuthZoneStackOffset, BucketOffset, ComponentOffset,
    FnIdentifier, GlobalOffset, KeyValueStoreOffset, PackageOffset, ProofOffset, RENodeId,
    ResourceManagerOffset, RoyaltyOffset, SubstateOffset, ValidatorOffset, VaultOffset,
    WorktopOffset,
};
use radix_engine_interface::blueprints::access_controller::ACCESS_CONTROLLER_BLUEPRINT;
use radix_engine_interface::blueprints::account::ACCOUNT_BLUEPRINT;
use radix_engine_interface::blueprints::clock::CLOCK_BLUEPRINT;
use radix_engine_interface::blueprints::epoch_manager::EPOCH_MANAGER_BLUEPRINT;
use radix_engine_interface::blueprints::identity::IDENTITY_BLUEPRINT;
use radix_engine_interface::constants::*;
use sbor::rust::collections::BTreeMap;

pub struct VisibilityProperties;

impl VisibilityProperties {
    pub fn check_drop_node_visibility(
        mode: ExecutionMode,
        actor: &ResolvedActor,
        node_id: RENodeId,
    ) -> bool {
        match mode {
            ExecutionMode::KernelModule => match node_id {
                RENodeId::Logger => true,
                RENodeId::EventStore => true,
                RENodeId::TransactionRuntime => true,
                RENodeId::AuthZoneStack => true,
                _ => false,
            },
            ExecutionMode::Client => match node_id {
                RENodeId::Worktop => match &actor.identifier {
                    FnIdentifier {
                        package_address,
                        blueprint_name,
                        ..
                    } if package_address.eq(&PACKAGE_LOADER)
                        && blueprint_name.eq(&TRANSACTION_PROCESSOR_BLUEPRINT) =>
                    {
                        true
                    }
                    _ => false,
                },
                RENodeId::Bucket(..) => match &actor.identifier {
                    FnIdentifier {
                        package_address: RESOURCE_MANAGER_PACKAGE,
                        ..
                    } => true,
                    _ => false,
                },
                RENodeId::Proof(..) => match &actor.identifier {
                    FnIdentifier {
                        package_address: RESOURCE_MANAGER_PACKAGE | AUTH_ZONE_PACKAGE,
                        ..
                    } => true,
                    FnIdentifier {
                        package_address,
                        blueprint_name,
                        ..
                    } if package_address.eq(&PACKAGE_LOADER)
                        && blueprint_name.eq(&TRANSACTION_PROCESSOR_BLUEPRINT) =>
                    {
                        true
                    }
                    _ => true,
                },
                _ => false,
            },
            _ => return false,
        }
    }

    pub fn check_create_node_access(
        mode: ExecutionMode,
        actor: &ResolvedActor,
        node: &RENodeInit,
        module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> bool {
        // TODO: Cleanup and reduce to least privilege
        match (mode, &actor.identifier) {
            (
                ExecutionMode::Client,
                FnIdentifier {
                    package_address,
                    blueprint_name,
                    ..
                },
            ) => match node {
                RENodeInit::Component(..) => {
                    if let Some(RENodeModuleInit::TypeInfo(type_info)) =
                        module_init.get(&NodeModuleId::TypeInfo)
                    {
                        blueprint_name.eq(&type_info.blueprint_name)
                            && package_address.eq(&type_info.package_address)
                    } else {
                        false
                    }
                }
                RENodeInit::Worktop(..) | RENodeInit::Package(..) => {
                    package_address.eq(&PACKAGE_LOADER)
                }
                RENodeInit::ResourceManager(..)
                | RENodeInit::Vault(..)
                | RENodeInit::Bucket(..)
                | RENodeInit::NonFungibleStore(..)
                | RENodeInit::Proof(..) => {
                    package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                        || package_address.eq(&AUTH_ZONE_PACKAGE)
                } // TODO: Remove AuthZonePackage
                RENodeInit::Identity() => {
                    package_address.eq(&IDENTITY_PACKAGE) && blueprint_name.eq(IDENTITY_BLUEPRINT)
                }
                RENodeInit::EpochManager(..) => {
                    package_address.eq(&EPOCH_MANAGER_PACKAGE)
                        && blueprint_name.eq(EPOCH_MANAGER_BLUEPRINT)
                }
                RENodeInit::Validator(..) => {
                    package_address.eq(&EPOCH_MANAGER_PACKAGE)
                        && blueprint_name.eq(EPOCH_MANAGER_BLUEPRINT)
                }
                RENodeInit::Clock(..) => {
                    package_address.eq(&CLOCK_PACKAGE) && blueprint_name.eq(CLOCK_BLUEPRINT)
                }
                RENodeInit::Account(..) => {
                    package_address.eq(&ACCOUNT_PACKAGE) && blueprint_name.eq(ACCOUNT_BLUEPRINT)
                }
                RENodeInit::AccessController(..) => {
                    package_address.eq(&ACCESS_CONTROLLER_PACKAGE)
                        && blueprint_name.eq(ACCESS_CONTROLLER_BLUEPRINT)
                }
                RENodeInit::KeyValueStore => true,
                RENodeInit::Global(..) => true,
                _ => false,
            },
            _ => true,
        }
    }

    pub fn check_substate_access(
        mode: ExecutionMode,
        actor: &ResolvedActor,
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> bool {
        let read_only = flags == LockFlags::read_only();

        // TODO: Cleanup and reduce to least privilege
        match (mode, offset) {
            (ExecutionMode::Kernel, offset) => match offset {
                SubstateOffset::Global(GlobalOffset::Global) => read_only,
                _ => false, // Protect ourselves!
            },
            (ExecutionMode::Resolver, offset) => match offset {
                SubstateOffset::Global(GlobalOffset::Global) => read_only,
                SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo) => read_only,
                SubstateOffset::Package(PackageOffset::CodeType) => read_only,
                SubstateOffset::Package(PackageOffset::Info) => read_only,
                SubstateOffset::Bucket(BucketOffset::Bucket) => read_only,
                _ => false,
            },
            (ExecutionMode::DropNode, offset) => match offset {
                SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo) => true,
                SubstateOffset::Bucket(BucketOffset::Bucket) => true,
                SubstateOffset::Proof(ProofOffset::Proof) => true,
                SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack) => true,
                SubstateOffset::Worktop(WorktopOffset::Worktop) => true,
                _ => false,
            },
            (ExecutionMode::KernelModule, offset) => match offset {
                // TODO: refine based on specific module
                SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack) => true,
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager) => {
                    read_only
                }
                SubstateOffset::Bucket(BucketOffset::Bucket) => read_only,
                SubstateOffset::Proof(ProofOffset::Proof) => true,
                SubstateOffset::Vault(VaultOffset::Vault) => true,
                SubstateOffset::Global(GlobalOffset::Global) => read_only,
                SubstateOffset::Package(PackageOffset::Info) => read_only,
                SubstateOffset::Package(PackageOffset::CodeType) => read_only,
                SubstateOffset::Package(PackageOffset::Code) => read_only,
                SubstateOffset::Component(ComponentOffset::State0) => read_only,
                SubstateOffset::PackageAccessRules => read_only,
                SubstateOffset::TypeInfo(_) => read_only,
                SubstateOffset::AccessRulesChain(_) => read_only,
                SubstateOffset::Royalty(_) => true,
                _ => false,
            },
            (ExecutionMode::Client, offset) => {
                if !flags.contains(LockFlags::MUTABLE) {
                    if matches!(offset, SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo)) {
                        return true;
                    }

                    match &actor.identifier {
                        // Native
                        FnIdentifier {
                            package_address, ..
                        } if package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                            || package_address.eq(&IDENTITY_PACKAGE)
                            || package_address.eq(&EPOCH_MANAGER_PACKAGE)
                            || package_address.eq(&CLOCK_PACKAGE)
                            || package_address.eq(&ACCOUNT_PACKAGE)
                            || package_address.eq(&LOGGER_PACKAGE)
                            || package_address.eq(&ACCESS_CONTROLLER_PACKAGE)
                            || package_address.eq(&TRANSACTION_RUNTIME_PACKAGE)
                            || package_address.eq(&AUTH_ZONE_PACKAGE)
                            || package_address.eq(&METADATA_PACKAGE)
                            || package_address.eq(&ROYALTY_PACKAGE)
                            || package_address.eq(&ACCESS_RULES_PACKAGE)
                            || package_address.eq(&PACKAGE_LOADER) =>
                        {
                            true
                        }
                        // Scrypto
                        _ => match &actor.receiver {
                            None => match (node_id, offset) {
                                // READ package code & abi
                                (
                                    RENodeId::Package(_),
                                    SubstateOffset::Package(PackageOffset::Info), // TODO: Remove
                                )
                                | (
                                    RENodeId::Package(_),
                                    SubstateOffset::Package(PackageOffset::CodeType), // TODO: Remove
                                )
                                | (
                                    RENodeId::Package(_),
                                    SubstateOffset::Package(PackageOffset::Code), // TODO: Remove
                                ) => read_only,
                                // READ global substates
                                (RENodeId::Global(_), SubstateOffset::Global(_)) => read_only,
                                (
                                    RENodeId::Component(_),
                                    SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
                                ) => read_only,
                                // READ/WRITE KVStore entry
                                (
                                    RENodeId::KeyValueStore(_),
                                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                ) => true,
                                // Otherwise, false
                                _ => false,
                            },
                            Some(ResolvedReceiver {
                                receiver: MethodReceiver(RENodeId::Component(component_address), ..),
                                ..
                            }) => match (node_id, offset) {
                                // READ package code & abi
                                (
                                    RENodeId::Package(_),
                                    SubstateOffset::Package(PackageOffset::Info), // TODO: Remove
                                )
                                | (
                                    RENodeId::Package(_),
                                    SubstateOffset::Package(PackageOffset::CodeType), // TODO: Remove
                                )
                                | (
                                    RENodeId::Package(_),
                                    SubstateOffset::Package(PackageOffset::Code), // TODO: Remove
                                ) => read_only,
                                // READ global substates
                                (RENodeId::Global(_), SubstateOffset::Global(_)) => read_only,
                                (
                                    RENodeId::Component(_),
                                    SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
                                ) => read_only,
                                // READ/WRITE KVStore entry
                                (
                                    RENodeId::KeyValueStore(_),
                                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                ) => true,
                                // READ/WRITE component application state
                                (
                                    RENodeId::Component(addr),
                                    SubstateOffset::Component(ComponentOffset::State0),
                                ) => addr.eq(component_address),
                                // Otherwise, false
                                _ => false,
                            },
                            _ => false,
                        },
                    }
                } else {
                    match &actor.identifier {
                        // Native
                        FnIdentifier {
                            package_address, ..
                        } if package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                            || package_address.eq(&IDENTITY_PACKAGE)
                            || package_address.eq(&ACCESS_CONTROLLER_PACKAGE)
                            || package_address.eq(&CLOCK_PACKAGE)
                            || package_address.eq(&LOGGER_PACKAGE)
                            || package_address.eq(&EPOCH_MANAGER_PACKAGE)
                            || package_address.eq(&TRANSACTION_RUNTIME_PACKAGE)
                            || package_address.eq(&AUTH_ZONE_PACKAGE)
                            || package_address.eq(&METADATA_PACKAGE)
                            || package_address.eq(&ROYALTY_PACKAGE)
                            || package_address.eq(&ACCESS_RULES_PACKAGE)
                            || package_address.eq(&PACKAGE_LOADER) =>
                        {
                            true
                        }

                        // Scrypto
                        _ => match &actor.receiver {
                            None => match (node_id, offset) {
                                (
                                    RENodeId::KeyValueStore(_),
                                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                ) => true,
                                _ => false,
                            },

                            Some(ResolvedReceiver {
                                receiver: MethodReceiver(RENodeId::Component(component_address), ..),
                                ..
                            })
                            | Some(ResolvedReceiver {
                                receiver: MethodReceiver(RENodeId::Account(component_address), ..),
                                ..
                            }) => match (node_id, offset) {
                                (
                                    RENodeId::KeyValueStore(_),
                                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                ) => true,
                                (
                                    RENodeId::Component(addr),
                                    SubstateOffset::Component(ComponentOffset::State0),
                                ) => addr.eq(component_address),
                                _ => false,
                            },
                            _ => false,
                        },
                    }
                }
            }
        }
    }
}

pub struct SubstateProperties;

impl SubstateProperties {
    pub fn is_persisted(offset: &SubstateOffset) -> bool {
        match offset {
            SubstateOffset::Global(..) => true,
            SubstateOffset::AuthZoneStack(..) => false,
            SubstateOffset::Component(..) => true,
            SubstateOffset::Royalty(..) => true,
            SubstateOffset::AccessRulesChain(..) => true,
            SubstateOffset::Metadata(..) => true,
            SubstateOffset::Package(..) => true,
            SubstateOffset::ResourceManager(..) => true,
            SubstateOffset::KeyValueStore(..) => true,
            SubstateOffset::NonFungibleStore(..) => true,
            SubstateOffset::Vault(..) => true,
            SubstateOffset::EpochManager(..) => true,
            SubstateOffset::Validator(..) => true,
            SubstateOffset::Bucket(..) => false,
            SubstateOffset::Proof(..) => false,
            SubstateOffset::Worktop(..) => false,
            SubstateOffset::Logger(..) => false,
            SubstateOffset::Clock(..) => true,
            SubstateOffset::TransactionRuntime(..) => false,
            SubstateOffset::Account(..) => true,
            SubstateOffset::AccessController(..) => true,
            SubstateOffset::TypeInfo(..) => true,
            SubstateOffset::PackageAccessRules => true,
            SubstateOffset::EventStore(..) => false,
        }
    }

    pub fn verify_can_own(offset: &SubstateOffset, node_id: RENodeId) -> Result<(), RuntimeError> {
        match offset {
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..))
            | SubstateOffset::Component(ComponentOffset::State0) => match node_id {
                RENodeId::KeyValueStore(..) | RENodeId::Component { .. } | RENodeId::Vault(..) => {
                    Ok(())
                }
                _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                    offset.clone(),
                    node_id,
                ))),
            },
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager) => {
                match node_id {
                    RENodeId::NonFungibleStore(..) => Ok(()),
                    _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                        offset.clone(),
                        node_id,
                    ))),
                }
            }
            SubstateOffset::Worktop(WorktopOffset::Worktop) => match node_id {
                RENodeId::Bucket(..) => Ok(()),
                _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                    offset.clone(),
                    node_id,
                ))),
            },
            SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator) => match node_id {
                RENodeId::Vault(..) => Ok(()),
                _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                    offset.clone(),
                    node_id,
                ))),
            },
            SubstateOffset::AccessController(AccessControllerOffset::AccessController) => {
                match node_id {
                    RENodeId::Vault(..) => Ok(()),
                    _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                        offset.clone(),
                        node_id,
                    ))),
                }
            }
            SubstateOffset::Validator(ValidatorOffset::Validator) => match node_id {
                RENodeId::Vault(..) => Ok(()),
                _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                    offset.clone(),
                    node_id,
                ))),
            },
            SubstateOffset::Account(AccountOffset::Account) => match node_id {
                RENodeId::KeyValueStore(..) => Ok(()),
                _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                    offset.clone(),
                    node_id,
                ))),
            },
            SubstateOffset::Global(GlobalOffset::Global) => match node_id {
                RENodeId::Component(..)
                | RENodeId::Package(..)
                | RENodeId::ResourceManager(..)
                | RENodeId::EpochManager(..)
                | RENodeId::Validator(..)
                | RENodeId::Clock(..)
                | RENodeId::Identity(..)
                | RENodeId::Account(..)
                | RENodeId::AccessController(..) => Ok(()),
                _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                    offset.clone(),
                    node_id,
                ))),
            },
            _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                offset.clone(),
                node_id,
            ))),
        }
    }
}
