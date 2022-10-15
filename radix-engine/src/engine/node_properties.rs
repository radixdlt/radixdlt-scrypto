use crate::engine::{
    KernelActor, KernelError, REActor, ResolvedFunction, ResolvedMethod, ResolvedReceiver,
    ResolvedReceiverMethod, RuntimeError,
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

    pub fn is_substate_readable(
        kernel_actor: KernelActor,
        actor: &REActor,
        node_id: RENodeId,
        offset: SubstateOffset,
    ) -> bool {
        match (kernel_actor, offset) {
            (KernelActor::Deref, offset) => match offset {
                SubstateOffset::Global(GlobalOffset::Global) => true,
                _ => false,
            },
            (KernelActor::Application, offset) => match actor {
                REActor::Function(ResolvedFunction::Native(..))
                | REActor::Method(ResolvedReceiverMethod {
                    method: ResolvedMethod::Native(..),
                    ..
                }) => true,
                REActor::Function(ResolvedFunction::Scrypto { .. }) => match (node_id, offset) {
                    (
                        RENodeId::KeyValueStore(_),
                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                    ) => true,
                    (RENodeId::Component(_), SubstateOffset::Component(ComponentOffset::Info)) => {
                        true
                    }
                    _ => false,
                },
                REActor::Method(ResolvedReceiverMethod {
                    receiver:
                        ResolvedReceiver {
                            receiver: Receiver::Ref(RENodeId::Component(component_address)),
                            ..
                        },
                    method: ResolvedMethod::Scrypto { .. },
                }) => match (node_id, offset) {
                    (
                        RENodeId::KeyValueStore(_),
                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                    ) => true,
                    (RENodeId::Component(_), SubstateOffset::Component(ComponentOffset::Info)) => {
                        true
                    }
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

    pub fn is_substate_writeable(
        kernel_actor: KernelActor,
        actor: &REActor,
        node_id: RENodeId,
        offset: SubstateOffset,
    ) -> bool {
        match actor {
            REActor::Function(ResolvedFunction::Native(..))
            | REActor::Method(ResolvedReceiverMethod {
                method: ResolvedMethod::Native(..),
                ..
            }) => true,
            REActor::Function(ResolvedFunction::Scrypto { .. }) => match (node_id, offset) {
                (
                    RENodeId::KeyValueStore(_),
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => true,
                _ => false,
            },
            REActor::Method(ResolvedReceiverMethod {
                receiver:
                    ResolvedReceiver {
                        receiver: Receiver::Ref(RENodeId::Component(component_address)),
                        ..
                    },
                method: ResolvedMethod::Scrypto { .. },
            }) => match (node_id, offset) {
                (
                    RENodeId::KeyValueStore(_),
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)),
                ) => true,
                (RENodeId::Component(addr), SubstateOffset::Component(ComponentOffset::State)) => {
                    addr.eq(component_address)
                }
                _ => false,
            },
            _ => false,
        }
    }
}
