use crate::engine::{
    ExecutionMode, KernelError, LockFlags, REActor, RENode, ResolvedFunction, ResolvedMethod,
    ResolvedReceiver, RuntimeError,
};
use crate::model::GlobalAddressSubstate;
use radix_engine_interface::api::types::{
    AccessRulesOffset, AuthZoneOffset, BucketOffset, ComponentOffset, GlobalOffset,
    KeyValueStoreOffset, NativeFunction, NativeMethod, PackageOffset, ProofOffset, RENodeId,
    ResourceManagerOffset, SubstateOffset, TransactionProcessorFunction, VaultOffset,
    WorktopOffset,
};

pub struct VisibilityProperties;

impl VisibilityProperties {
    pub fn check_drop_node_visibility(
        mode: ExecutionMode,
        actor: &REActor,
        node_id: RENodeId,
    ) -> bool {
        if !mode.eq(&ExecutionMode::Application) {
            return false;
        }

        // TODO: Cleanup and reduce to least privilege
        match node_id {
            RENodeId::Worktop => match actor {
                REActor::Function(
                    ResolvedFunction::Native(NativeFunction::TransactionProcessor(..)),
                    ..,
                ) => true,
                _ => false,
            },
            RENodeId::AuthZoneStack(..) => match actor {
                REActor::Function(
                    ResolvedFunction::Native(NativeFunction::TransactionProcessor(..)),
                    ..,
                ) => true,
                _ => false,
            },
            RENodeId::Bucket(..) => match actor {
                REActor::Method(ResolvedMethod::Native(NativeMethod::Bucket(..)), ..)
                | REActor::Method(ResolvedMethod::Native(NativeMethod::Worktop(..)), ..)
                | REActor::Method(ResolvedMethod::Native(NativeMethod::ResourceManager(..)), ..)
                | REActor::Method(ResolvedMethod::Native(NativeMethod::Vault(..)), ..) => true,
                _ => false,
            },
            RENodeId::Proof(..) => match actor {
                REActor::Method(ResolvedMethod::Native(NativeMethod::AuthZone(..)), ..) => true,
                REActor::Method(ResolvedMethod::Native(NativeMethod::Proof(..)), ..) => true,
                REActor::Function(ResolvedFunction::Native(
                    NativeFunction::TransactionProcessor(TransactionProcessorFunction::Run),
                )) => true,
                REActor::Method(ResolvedMethod::Scrypto { .. }, ..) => true,
                REActor::Function(ResolvedFunction::Scrypto { .. }) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn check_create_node_visibility(
        mode: ExecutionMode,
        actor: &REActor,
        node: &RENode,
    ) -> bool {
        // TODO: Cleanup and reduce to least privilege
        match (mode, actor) {
            (
                ExecutionMode::Application,
                REActor::Method(
                    ResolvedMethod::Scrypto {
                        package_address,
                        blueprint_name,
                        ..
                    },
                    ..,
                )
                | REActor::Function(
                    ResolvedFunction::Scrypto {
                        package_address,
                        blueprint_name,
                        ..
                    },
                    ..,
                ),
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
        actor: &REActor,
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
            (ExecutionMode::NodeMoveModule, offset) => match offset {
                SubstateOffset::Bucket(BucketOffset::Bucket) => flags == LockFlags::read_only(),
                SubstateOffset::Proof(ProofOffset::Proof) => true,
                _ => false,
            },
            (ExecutionMode::MoveUpstream, offset) => match offset {
                SubstateOffset::Bucket(BucketOffset::Bucket) => flags == LockFlags::read_only(),
                _ => false,
            },
            (ExecutionMode::DropNode, offset) => match offset {
                SubstateOffset::Bucket(BucketOffset::Bucket) => true,
                SubstateOffset::Proof(ProofOffset::Proof) => true,
                SubstateOffset::AuthZone(AuthZoneOffset::AuthZone) => true,
                SubstateOffset::Worktop(WorktopOffset::Worktop) => true,
                _ => false,
            },
            (ExecutionMode::AuthModule, offset) => match offset {
                SubstateOffset::AuthZone(AuthZoneOffset::AuthZone) => true,
                // TODO: Remove these and use AuthRulesSubstate
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager) => {
                    flags == LockFlags::read_only()
                }
                SubstateOffset::Bucket(BucketOffset::Bucket) => true, // TODO: Remove to read_only!
                SubstateOffset::Vault(VaultOffset::Vault) => flags == LockFlags::read_only(),
                SubstateOffset::Package(PackageOffset::Package) => flags == LockFlags::read_only(),
                SubstateOffset::Component(ComponentOffset::State) => {
                    flags == LockFlags::read_only()
                }
                SubstateOffset::Component(ComponentOffset::Info) => flags == LockFlags::read_only(),
                SubstateOffset::AccessRules(AccessRulesOffset::AccessRules) => {
                    flags == LockFlags::read_only()
                }
                _ => false,
            },
            (ExecutionMode::ScryptoInterpreter, offset) => match offset {
                SubstateOffset::Global(GlobalOffset::Global) => flags == LockFlags::read_only(),
                SubstateOffset::Component(ComponentOffset::Info) => flags == LockFlags::read_only(),
                SubstateOffset::Package(PackageOffset::Package) => flags == LockFlags::read_only(),
                _ => false,
            },
            (ExecutionMode::Application, offset) => {
                if !flags.contains(LockFlags::MUTABLE) {
                    match actor {
                        // Native
                        REActor::Function(ResolvedFunction::Native(..))
                        | REActor::Method(ResolvedMethod::Native(..), ..) => true,

                        // Scrypto
                        REActor::Function(ResolvedFunction::Scrypto { .. }) => {
                            match (node_id, offset) {
                                (
                                    RENodeId::KeyValueStore(_),
                                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                ) => true,
                                (
                                    RENodeId::Component(_),
                                    SubstateOffset::Component(ComponentOffset::Info),
                                ) => true,
                                (
                                    RENodeId::Component(_),
                                    SubstateOffset::AccessRules(AccessRulesOffset::AccessRules),
                                ) => true,
                                _ => false,
                            }
                        }
                        REActor::Method(
                            ResolvedMethod::Scrypto { .. },
                            ResolvedReceiver {
                                receiver: RENodeId::Component(component_address),
                                ..
                            },
                        ) => match (node_id, offset) {
                            (
                                RENodeId::KeyValueStore(_),
                                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                            ) => true,
                            (
                                RENodeId::Component(_),
                                SubstateOffset::Component(ComponentOffset::Info),
                            ) => true,
                            (
                                RENodeId::Component(_),
                                SubstateOffset::AccessRules(AccessRulesOffset::AccessRules),
                            ) => true,
                            (
                                RENodeId::Component(addr),
                                SubstateOffset::Component(ComponentOffset::State),
                            ) => addr.eq(component_address),
                            _ => false,
                        },
                        _ => false,
                    }
                } else {
                    match actor {
                        // Native
                        REActor::Function(ResolvedFunction::Native(..))
                        | REActor::Method(ResolvedMethod::Native(..), ..) => true,
                        REActor::Function(ResolvedFunction::Scrypto { .. }) => {
                            match (node_id, offset) {
                                (
                                    RENodeId::KeyValueStore(_),
                                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                                ) => true,
                                _ => false,
                            }
                        }

                        // Scrypto
                        REActor::Method(
                            ResolvedMethod::Scrypto { .. },
                            ResolvedReceiver {
                                receiver: RENodeId::Component(component_address),
                                ..
                            },
                        ) => match (node_id, offset) {
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
            _ => Err(RuntimeError::KernelError(KernelError::InvalidOwnership(
                offset.clone(),
                node_id,
            ))),
        }
    }
}
