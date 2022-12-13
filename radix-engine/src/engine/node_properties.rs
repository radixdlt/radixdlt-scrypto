use crate::engine::{
    ExecutionMode, KernelError, LockFlags, RENode, ResolvedActor, ResolvedReceiver, RuntimeError,
};
use crate::model::GlobalAddressSubstate;
use radix_engine_interface::api::types::{
    AccessRulesChainOffset, AuthZoneStackOffset, BucketOffset, ComponentOffset, FnIdentifier,
    GlobalOffset, KeyValueStoreOffset, NativeFunction, NativeMethod, PackageOffset, ProofOffset,
    RENodeId, ResourceManagerOffset, ScryptoFnIdentifier, SubstateOffset,
    TransactionProcessorFunction, VaultOffset, WorktopOffset,
};

pub struct VisibilityProperties;

impl VisibilityProperties {
    pub fn check_drop_node_visibility(
        mode: ExecutionMode,
        actor: &ResolvedActor,
        node_id: RENodeId,
    ) -> bool {
        match mode {
            ExecutionMode::LoggerModule => match node_id {
                RENodeId::Logger => return true,
                _ => return false,
            }
            ExecutionMode::Application => {},
            _ => return false,
        }

        // TODO: Cleanup and reduce to least privilege
        match node_id {
            RENodeId::Worktop => match &actor.identifier {
                FnIdentifier::NativeFunction(NativeFunction::TransactionProcessor(..)) => true,
                _ => false,
            },
            RENodeId::AuthZoneStack(..) => match &actor.identifier {
                FnIdentifier::NativeFunction(NativeFunction::TransactionProcessor(..)) => true,
                _ => false,
            },
            RENodeId::TransactionHash(..) => match &actor.identifier {
                FnIdentifier::NativeFunction(NativeFunction::TransactionProcessor(..)) => true,
                _ => false,
            },
            RENodeId::Bucket(..) => match &actor.identifier {
                FnIdentifier::NativeMethod(NativeMethod::Bucket(..))
                | FnIdentifier::NativeMethod(NativeMethod::Worktop(..))
                | FnIdentifier::NativeMethod(NativeMethod::ResourceManager(..))
                | FnIdentifier::NativeMethod(NativeMethod::Vault(..)) => true,
                _ => false,
            },
            RENodeId::Proof(..) => match &actor.identifier {
                FnIdentifier::NativeMethod(NativeMethod::AuthZoneStack(..)) => true,
                FnIdentifier::NativeMethod(NativeMethod::Proof(..)) => true,
                FnIdentifier::NativeFunction(NativeFunction::TransactionProcessor(
                    TransactionProcessorFunction::Run,
                )) => true,
                FnIdentifier::Scrypto(..) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn check_create_node_visibility(
        mode: ExecutionMode,
        actor: &ResolvedActor,
        node: &RENode,
    ) -> bool {
        // TODO: Cleanup and reduce to least privilege
        match (mode, &actor.identifier) {
            (
                ExecutionMode::Application,
                FnIdentifier::Scrypto(ScryptoFnIdentifier {
                    package_address,
                    blueprint_name,
                    ..
                }),
            ) => match node {
                RENode::Component(info, ..) => {
                    blueprint_name.eq(&info.blueprint_name)
                        && package_address.eq(&info.package_address)
                }
                RENode::KeyValueStore(..) => true,
                RENode::Global(GlobalAddressSubstate::Component(..)) => true,
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
        // TODO: Cleanup and reduce to least privilege
        match (mode, offset) {
            (ExecutionMode::Kernel, ..) => false, // Protect ourselves!
            (ExecutionMode::Deref, offset) => match offset {
                SubstateOffset::Global(GlobalOffset::Global) => flags == LockFlags::read_only(),
                _ => false,
            },
            (ExecutionMode::Globalize, offset) => match offset {
                SubstateOffset::Component(ComponentOffset::Info) => flags == LockFlags::read_only(),
                _ => false,
            },
            (ExecutionMode::LoggerModule, ..) => false,
            (ExecutionMode::NodeMoveModule, offset) => match offset {
                SubstateOffset::Bucket(BucketOffset::Bucket) => flags == LockFlags::read_only(),
                SubstateOffset::Proof(ProofOffset::Proof) => true,
                _ => false,
            },
            (ExecutionMode::TransactionModule, _offset) => false,
            (ExecutionMode::MoveUpstream, offset) => match offset {
                SubstateOffset::Bucket(BucketOffset::Bucket) => flags == LockFlags::read_only(),
                _ => false,
            },
            (ExecutionMode::DropNode, offset) => match offset {
                SubstateOffset::Bucket(BucketOffset::Bucket) => true,
                SubstateOffset::Proof(ProofOffset::Proof) => true,
                SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack) => true,
                SubstateOffset::Worktop(WorktopOffset::Worktop) => true,
                _ => false,
            },
            (ExecutionMode::EntityModule, _offset) => false,
            (ExecutionMode::AuthModule, offset) => match offset {
                SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack) => true,
                // TODO: Remove these and use AuthRulesSubstate
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager) => {
                    flags == LockFlags::read_only()
                }
                SubstateOffset::Bucket(BucketOffset::Bucket) => true, // TODO: Remove to read_only!
                SubstateOffset::Vault(VaultOffset::Vault) => flags == LockFlags::read_only(),
                SubstateOffset::Package(PackageOffset::Info) => flags == LockFlags::read_only(),
                SubstateOffset::Component(ComponentOffset::State) => {
                    flags == LockFlags::read_only()
                }
                SubstateOffset::Component(ComponentOffset::Info) => flags == LockFlags::read_only(),
                SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain) => {
                    flags == LockFlags::read_only()
                }
                SubstateOffset::VaultAccessRulesChain(AccessRulesChainOffset::AccessRulesChain) => {
                    flags == LockFlags::read_only()
                }
                _ => false,
            },
            (ExecutionMode::ScryptoInterpreter, offset) => match offset {
                SubstateOffset::Global(GlobalOffset::Global) => flags == LockFlags::read_only(),
                SubstateOffset::Component(ComponentOffset::Info) => flags == LockFlags::read_only(),
                SubstateOffset::Package(PackageOffset::Info) => flags == LockFlags::read_only(),
                _ => false,
            },
            (ExecutionMode::Application, offset) => {
                if !flags.contains(LockFlags::MUTABLE) {
                    match &actor.identifier {
                        // Native
                        FnIdentifier::NativeMethod(..) | FnIdentifier::NativeFunction(..) => true,
                        // Scrypto
                        FnIdentifier::Scrypto(..) => match &actor.receiver {
                            None => match (node_id, offset) {
                                (
                                    RENodeId::KeyValueStore(_),
                                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                ) => true,
                                (
                                    RENodeId::Component(_),
                                    SubstateOffset::Component(ComponentOffset::Info),
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
                                    RENodeId::Component(_),
                                    SubstateOffset::Component(ComponentOffset::Info),
                                ) => true,
                                (
                                    RENodeId::Component(addr),
                                    SubstateOffset::Component(ComponentOffset::State),
                                ) => addr.eq(component_address),
                                _ => false,
                            },
                            _ => false,
                        },
                    }
                } else {
                    match &actor.identifier {
                        // Native
                        FnIdentifier::NativeMethod(..) | FnIdentifier::NativeFunction(..) => true,

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
                                    SubstateOffset::Component(ComponentOffset::State),
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
    pub fn verify_can_own(offset: &SubstateOffset, node_id: RENodeId) -> Result<(), RuntimeError> {
        match offset {
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..))
            | SubstateOffset::Component(ComponentOffset::State) => match node_id {
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
            SubstateOffset::Global(GlobalOffset::Global) => match node_id {
                RENodeId::Component(..)
                | RENodeId::Package(..)
                | RENodeId::ResourceManager(..)
                | RENodeId::EpochManager(..)
                | RENodeId::Clock(..) => Ok(()),
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
