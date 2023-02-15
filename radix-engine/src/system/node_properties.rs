use super::node::{RENodeInit, RENodeModuleInit};
use crate::errors::{KernelError, RuntimeError};
use crate::kernel::actor::{ExecutionMode, ResolvedActor, ResolvedReceiver};
use crate::kernel::kernel_api::LockFlags;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{
    AccessControllerOffset, AccountOffset, AuthZoneStackOffset, BucketOffset, ComponentOffset,
    FnIdentifier, GlobalOffset, KeyValueStoreOffset, NativeFn, PackageOffset, ProofOffset,
    RENodeId, ResourceManagerOffset, RoyaltyOffset, ScryptoFnIdentifier, SubstateOffset,
    TransactionProcessorFn, ValidatorOffset, VaultOffset, WorktopOffset,
};
use radix_engine_interface::blueprints::access_controller::ACCESS_CONTROLLER_BLUEPRINT;
use radix_engine_interface::blueprints::account::ACCOUNT_BLUEPRINT;
use radix_engine_interface::blueprints::clock::CLOCK_BLUEPRINT;
use radix_engine_interface::blueprints::epoch_manager::EPOCH_MANAGER_BLUEPRINT;
use radix_engine_interface::blueprints::identity::IDENTITY_BLUEPRINT;
use radix_engine_interface::blueprints::resource::RESOURCE_MANAGER_BLUEPRINT;
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
                RENodeId::Logger => match &actor.identifier {
                    FnIdentifier::Native(NativeFn::Root) => true,
                    _ => false,
                },
                RENodeId::TransactionRuntime => match &actor.identifier {
                    FnIdentifier::Native(NativeFn::Root) => true,
                    _ => false,
                },
                RENodeId::AuthZoneStack => match &actor.identifier {
                    FnIdentifier::Native(NativeFn::Root) => true,
                    _ => false,
                },
                _ => false,
            },
            ExecutionMode::Client => match node_id {
                RENodeId::Worktop => match &actor.identifier {
                    FnIdentifier::Native(NativeFn::TransactionProcessor(
                        TransactionProcessorFn::Run,
                    )) => true,
                    _ => false,
                },
                RENodeId::Bucket(..) => match &actor.identifier {
                    FnIdentifier::Native(NativeFn::Bucket(..))
                    | FnIdentifier::Native(NativeFn::Worktop(..))
                    | FnIdentifier::Native(NativeFn::ResourceManager(..))
                    | FnIdentifier::Native(NativeFn::Vault(..)) => true,
                    _ => false,
                },
                RENodeId::Proof(..) => match &actor.identifier {
                    FnIdentifier::Native(NativeFn::AuthZoneStack(..)) => true,
                    FnIdentifier::Native(NativeFn::Proof(..)) => true,
                    FnIdentifier::Native(NativeFn::TransactionProcessor(
                        TransactionProcessorFn::Run,
                    )) => true,
                    FnIdentifier::Scrypto(..) => true,
                    _ => false,
                },
                _ => false,
            },
            _ => return false,
        }
    }

    pub fn check_create_node_visibility(
        mode: ExecutionMode,
        actor: &ResolvedActor,
        node: &RENodeInit,
        module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> bool {
        // TODO: Cleanup and reduce to least privilege
        match (mode, &actor.identifier) {
            (
                ExecutionMode::Client,
                FnIdentifier::Scrypto(ScryptoFnIdentifier {
                    package_address,
                    blueprint_name,
                    ..
                }),
            ) => match node {
                RENodeInit::Component(..) => {
                    if let Some(RENodeModuleInit::ComponentTypeInfo(type_info)) =
                        module_init.get(&NodeModuleId::ComponentTypeInfo)
                    {
                        blueprint_name.eq(&type_info.blueprint_name)
                            && package_address.eq(&type_info.package_address)
                    } else {
                        false
                    }
                }
                RENodeInit::ResourceManager(..)
                | RENodeInit::Bucket(..)
                | RENodeInit::NonFungibleStore(..) => {
                    package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                        && blueprint_name.eq(RESOURCE_MANAGER_BLUEPRINT)
                }
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

    pub fn check_substate_visibility(
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
                SubstateOffset::ComponentTypeInfo(ComponentTypeInfoOffset::TypeInfo) => read_only,
                SubstateOffset::Package(PackageOffset::Info) => read_only,
                SubstateOffset::PackageTypeInfo => read_only,
                SubstateOffset::Bucket(BucketOffset::Bucket) => read_only,
                _ => false,
            },
            (ExecutionMode::DropNode, offset) => match offset {
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
                SubstateOffset::Component(ComponentOffset::State0) => read_only,
                SubstateOffset::ComponentTypeInfo(_) => read_only,
                SubstateOffset::AccessRulesChain(_) => read_only,
                SubstateOffset::Royalty(_) => true,
                _ => false,
            },
            (ExecutionMode::Client, offset) => {
                if !flags.contains(LockFlags::MUTABLE) {
                    if matches!(offset, SubstateOffset::PackageTypeInfo) {
                        return true;
                    }

                    match &actor.identifier {
                        // Native
                        FnIdentifier::Native(..) => true,
                        FnIdentifier::Scrypto(ScryptoFnIdentifier {
                            package_address, ..
                        }) if package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                            || package_address.eq(&IDENTITY_PACKAGE)
                            || package_address.eq(&EPOCH_MANAGER_PACKAGE)
                            || package_address.eq(&CLOCK_PACKAGE)
                            || package_address.eq(&ACCOUNT_PACKAGE)
                            || package_address.eq(&ACCESS_CONTROLLER_PACKAGE) =>
                        {
                            true
                        }
                        // Scrypto
                        FnIdentifier::Scrypto(..) => match &actor.receiver {
                            None => match (node_id, offset) {
                                // READ package code & abi
                                (
                                    RENodeId::Package(_),
                                    SubstateOffset::Package(PackageOffset::Info), // TODO: Remove
                                ) => read_only,
                                // READ global substates
                                (RENodeId::Global(_), SubstateOffset::Global(_)) => read_only,
                                (
                                    RENodeId::Component(_),
                                    SubstateOffset::ComponentTypeInfo(
                                        ComponentTypeInfoOffset::TypeInfo,
                                    ),
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
                                receiver: RENodeId::Component(component_address),
                                ..
                            }) => match (node_id, offset) {
                                // READ package code & abi
                                (
                                    RENodeId::Package(_),
                                    SubstateOffset::Package(PackageOffset::Info), // TODO: Remove
                                ) => read_only,
                                // READ global substates
                                (RENodeId::Global(_), SubstateOffset::Global(_)) => read_only,
                                (
                                    RENodeId::Component(_),
                                    SubstateOffset::ComponentTypeInfo(
                                        ComponentTypeInfoOffset::TypeInfo,
                                    ),
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
                        FnIdentifier::Native(..) => true,
                        FnIdentifier::Scrypto(ScryptoFnIdentifier {
                            package_address, ..
                        }) if package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                            || package_address.eq(&IDENTITY_PACKAGE) =>
                        {
                            true
                        }

                        // Scrypto
                        FnIdentifier::Scrypto(..) => match &actor.receiver {
                            None => match (node_id, offset) {
                                (
                                    RENodeId::KeyValueStore(_),
                                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                ) => true,
                                _ => false,
                            },

                            Some(ResolvedReceiver {
                                receiver: RENodeId::Component(component_address),
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
            SubstateOffset::PackageTypeInfo => true,
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
            SubstateOffset::ComponentTypeInfo(..) => true,
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
