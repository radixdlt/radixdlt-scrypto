use crate::engine::{
    ExecutionMode, KernelError, LockFlags, REActor, ResolvedFunction, ResolvedMethod,
    ResolvedReceiver, RuntimeError,
};
use crate::types::*;

pub struct NodeProperties;

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

    pub fn check_substate_access(
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
            (ExecutionMode::MoveDownstream, offset) => match offset {
                SubstateOffset::Bucket(BucketOffset::Bucket) => flags == LockFlags::read_only(),
                SubstateOffset::Proof(ProofOffset::Proof) => true,
                _ => false,
            },
            (ExecutionMode::MoveUpstream, offset) => match offset {
                SubstateOffset::Bucket(BucketOffset::Bucket) => flags == LockFlags::read_only(),
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
                                _ => false,
                            }
                        }
                        REActor::Method(
                            ResolvedMethod::Scrypto { .. },
                            ResolvedReceiver {
                                receiver: Receiver::Ref(RENodeId::Component(component_address)),
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
                                receiver: Receiver::Ref(RENodeId::Component(component_address)),
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
