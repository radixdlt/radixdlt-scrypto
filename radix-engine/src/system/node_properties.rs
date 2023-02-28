use crate::errors::{KernelError, RuntimeError};
use crate::kernel::actor::{Actor, ActorIdentifier, ExecutionMode};
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{
    AccessControllerOffset, AccountOffset, AuthZoneStackOffset, BucketOffset, ComponentOffset,
    FnIdentifier, KeyValueStoreOffset, PackageOffset, ProofOffset, RENodeId, ResourceManagerOffset,
    RoyaltyOffset, SubstateOffset, ValidatorOffset, WorktopOffset,
};
use radix_engine_interface::blueprints::resource::PROOF_BLUEPRINT;
use radix_engine_interface::constants::*;

pub struct VisibilityProperties;

impl VisibilityProperties {
    pub fn check_drop_node_visibility(
        mode: ExecutionMode,
        actor: &Actor,
        node_id: RENodeId,
    ) -> bool {
        match mode {
            ExecutionMode::Kernel => match node_id {
                // TODO: Remove
                RENodeId::Account(..) => true,
                RENodeId::Identity(..) => true,
                RENodeId::Component(..) => true,
                _ => false,
            },
            ExecutionMode::KernelModule => match node_id {
                RENodeId::Logger => true,
                RENodeId::TransactionRuntime => true,
                RENodeId::AuthZoneStack => true,
                _ => false,
            },
            ExecutionMode::Client | ExecutionMode::AutoDrop => match node_id {
                RENodeId::Worktop => match &actor.fn_identifier {
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
                RENodeId::Bucket(..) => match &actor.fn_identifier {
                    FnIdentifier {
                        package_address, ..
                    } if package_address.eq(&RESOURCE_MANAGER_PACKAGE) => true,
                    _ => false,
                },
                RENodeId::Proof(..) => match &actor.fn_identifier {
                    FnIdentifier {
                        package_address,
                        blueprint_name,
                        ..
                    } if package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                        && blueprint_name.eq(&PROOF_BLUEPRINT) =>
                    {
                        true
                    }
                    _ => false,
                },
                // TODO: CLEAN THESE UP, these are used for globalization
                RENodeId::Clock(..) => mode.eq(&ExecutionMode::Client),
                RENodeId::EpochManager(..) => mode.eq(&ExecutionMode::Client),
                RENodeId::Account(..) => mode.eq(&ExecutionMode::Client),
                RENodeId::Validator(..) => mode.eq(&ExecutionMode::Client),
                RENodeId::Component(..) => mode.eq(&ExecutionMode::Client),
                RENodeId::AccessController(..) => mode.eq(&ExecutionMode::Client),
                RENodeId::Identity(..) => mode.eq(&ExecutionMode::Client),
                _ => false,
            },
            _ => return false,
        }
    }

    pub fn check_substate_access(
        mode: ExecutionMode,
        actor: &Actor,
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> bool {
        let read_only = flags == LockFlags::read_only();

        // TODO: Cleanup and reduce to least privilege
        match (mode, offset) {
            (ExecutionMode::Kernel, offset) => match offset {
                _ => false, // Protect ourselves!
            },
            (ExecutionMode::Resolver, offset) => match offset {
                SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo) => read_only,
                SubstateOffset::Package(PackageOffset::CodeType) => read_only,
                SubstateOffset::Package(PackageOffset::Info) => read_only,
                SubstateOffset::Bucket(BucketOffset::Info) => read_only,
                _ => false,
            },
            (ExecutionMode::AutoDrop, offset) => match offset {
                _ => false,
            },
            (ExecutionMode::DropNode, offset) => match offset {
                SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo) => true,
                SubstateOffset::Bucket(BucketOffset::Info) => true,
                SubstateOffset::Proof(ProofOffset::Info) => true,
                SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack) => true,
                SubstateOffset::Proof(..) => true,
                SubstateOffset::Worktop(WorktopOffset::Worktop) => true,
                _ => false,
            },
            (ExecutionMode::KernelModule, offset) => match offset {
                // TODO: refine based on specific module
                SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack) => true,
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager) => {
                    read_only
                }
                SubstateOffset::Vault(..) => true,
                SubstateOffset::Bucket(..) => read_only,
                SubstateOffset::Proof(..) => true,
                SubstateOffset::Package(PackageOffset::Info) => read_only,
                SubstateOffset::Package(PackageOffset::CodeType) => read_only,
                SubstateOffset::Package(PackageOffset::Code) => read_only,
                SubstateOffset::Component(ComponentOffset::State0) => read_only,
                SubstateOffset::PackageAccessRules => read_only,
                SubstateOffset::TypeInfo(_) => read_only,
                SubstateOffset::AccessRules(_) => read_only,
                SubstateOffset::Royalty(_) => true,
                _ => false,
            },
            (ExecutionMode::Client, offset) => {
                if !flags.contains(LockFlags::MUTABLE) {
                    if matches!(offset, SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo)) {
                        return true;
                    }

                    match &actor.fn_identifier {
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
                        _ => match &actor.identifier {
                            ActorIdentifier::Function(..) => match (node_id, offset) {
                                // READ package code & abi
                                (
                                    RENodeId::GlobalPackage(_),
                                    SubstateOffset::Package(PackageOffset::Info), // TODO: Remove
                                )
                                | (
                                    RENodeId::GlobalPackage(_),
                                    SubstateOffset::Package(PackageOffset::CodeType), // TODO: Remove
                                )
                                | (
                                    RENodeId::GlobalPackage(_),
                                    SubstateOffset::Package(PackageOffset::Code), // TODO: Remove
                                ) => read_only,
                                // READ global substates
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
                            ActorIdentifier::Method(method_identifier) => match method_identifier {
                                MethodIdentifier(RENodeId::Component(component_address), ..) => {
                                    match (node_id, offset) {
                                        // READ package code & abi
                                        (
                                            RENodeId::GlobalPackage(_),
                                            SubstateOffset::Package(PackageOffset::Info), // TODO: Remove
                                        )
                                        | (
                                            RENodeId::GlobalPackage(_),
                                            SubstateOffset::Package(PackageOffset::CodeType), // TODO: Remove
                                        )
                                        | (
                                            RENodeId::GlobalPackage(_),
                                            SubstateOffset::Package(PackageOffset::Code), // TODO: Remove
                                        ) => read_only,
                                        // READ/WRITE KVStore entry
                                        (
                                            RENodeId::KeyValueStore(_),
                                            SubstateOffset::KeyValueStore(
                                                KeyValueStoreOffset::Entry(..),
                                            ),
                                        ) => true,
                                        // READ/WRITE component application state
                                        (
                                            RENodeId::Component(addr),
                                            SubstateOffset::Component(ComponentOffset::State0),
                                        ) => addr.eq(component_address),
                                        // Otherwise, false
                                        _ => false,
                                    }
                                }
                                MethodIdentifier(
                                    RENodeId::GlobalComponent(component_address),
                                    ..,
                                ) => match (node_id, offset) {
                                    // READ package code & abi
                                    (
                                        RENodeId::GlobalPackage(_),
                                        SubstateOffset::Package(PackageOffset::Info), // TODO: Remove
                                    )
                                    | (
                                        RENodeId::GlobalPackage(_),
                                        SubstateOffset::Package(PackageOffset::CodeType), // TODO: Remove
                                    )
                                    | (
                                        RENodeId::GlobalPackage(_),
                                        SubstateOffset::Package(PackageOffset::Code), // TODO: Remove
                                    ) => read_only,
                                    // READ/WRITE KVStore entry
                                    (
                                        RENodeId::KeyValueStore(_),
                                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
                                            ..,
                                        )),
                                    ) => true,
                                    // READ/WRITE component application state
                                    (
                                        RENodeId::GlobalComponent(addr),
                                        SubstateOffset::Component(ComponentOffset::State0),
                                    ) => addr.eq(component_address),
                                    // Otherwise, false
                                    _ => false,
                                },
                                _ => false,
                            },
                        },
                    }
                } else {
                    match &actor.fn_identifier {
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
                            || package_address.eq(&PACKAGE_LOADER)
                            || package_address.eq(&ACCOUNT_PACKAGE) =>
                        {
                            true
                        }

                        // Scrypto
                        _ => match &actor.identifier {
                            ActorIdentifier::Function(..) => match (node_id, offset) {
                                (
                                    RENodeId::KeyValueStore(_),
                                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                ) => true,
                                _ => false,
                            },

                            ActorIdentifier::Method(method_identifier) => match method_identifier {
                                MethodIdentifier(RENodeId::Component(component_address), ..)
                                | MethodIdentifier(RENodeId::Account(component_address), ..) => {
                                    match (node_id, offset) {
                                        (
                                            RENodeId::KeyValueStore(_),
                                            SubstateOffset::KeyValueStore(
                                                KeyValueStoreOffset::Entry(..),
                                            ),
                                        ) => true,
                                        (
                                            RENodeId::Component(addr),
                                            SubstateOffset::Component(ComponentOffset::State0),
                                        ) => addr.eq(component_address),
                                        _ => false,
                                    }
                                }
                                MethodIdentifier(
                                    RENodeId::GlobalComponent(component_address),
                                    ..,
                                ) => match (node_id, offset) {
                                    (
                                        RENodeId::KeyValueStore(_),
                                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
                                            ..,
                                        )),
                                    ) => true,
                                    (
                                        RENodeId::GlobalComponent(addr),
                                        SubstateOffset::Component(ComponentOffset::State0),
                                    ) => addr.eq(component_address),
                                    _ => false,
                                },
                                _ => false,
                            },
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
            SubstateOffset::AuthZoneStack(..) => false,
            SubstateOffset::Component(..) => true,
            SubstateOffset::Royalty(..) => true,
            SubstateOffset::AccessRules(..) => true,
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
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack) => match node_id {
                RENodeId::Proof(..) => Ok(()),
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
