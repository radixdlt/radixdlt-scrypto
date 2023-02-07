use crate::blueprints::fee_reserve::FeeReserveSubstate;
use crate::blueprints::logger::LoggerSubstate;
use crate::blueprints::resource::{BucketSubstate, NonFungibleSubstate, ProofSubstate};
use crate::blueprints::transaction_runtime::TransactionRuntimeSubstate;
use crate::errors::CallFrameError;
use crate::kernel::Track;
use crate::system::node_modules::auth::AuthZoneStackSubstate;
use crate::system::node_substates::{RuntimeSubstate, SubstateRef, SubstateRefMut};
use crate::types::{HashMap, HashSet};
use radix_engine_interface::api::component::KeyValueStoreEntrySubstate;
use radix_engine_interface::api::types::{
    AuthZoneStackOffset, BucketOffset, FeeReserveOffset, LoggerOffset, NodeModuleId, ProofOffset,
    RENodeId, SubstateId, SubstateOffset, TransactionRuntimeOffset,
};
use sbor::rust::collections::BTreeMap;

pub struct Heap {
    nodes: HashMap<RENodeId, HeapRENode>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn get_substate(
        &mut self,
        node_id: RENodeId,
        module_id: NodeModuleId,
        offset: &SubstateOffset,
    ) -> Result<SubstateRef, CallFrameError> {
        let node = self
            .nodes
            .get_mut(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))?;

        // TODO: Will clean this up when virtual substates is cleaned up
        match (&node_id, offset) {
            (RENodeId::KeyValueStore(..), SubstateOffset::KeyValueStore(..)) => {
                let entry = node.substates.entry((module_id, offset.clone())).or_insert(
                    RuntimeSubstate::KeyValueStoreEntry(KeyValueStoreEntrySubstate::None),
                );
                Ok(entry.to_ref())
            }
            (RENodeId::NonFungibleStore(..), SubstateOffset::NonFungibleStore(..)) => {
                let entry = node
                    .substates
                    .entry((module_id, offset.clone()))
                    .or_insert(RuntimeSubstate::NonFungible(NonFungibleSubstate(None)));
                Ok(entry.to_ref())
            }
            _ => node
                .substates
                .get(&(module_id, offset.clone()))
                .map(|s| s.to_ref())
                .ok_or(CallFrameError::OffsetDoesNotExist(node_id, offset.clone())),
        }
    }

    pub fn get_substate_mut(
        &mut self,
        node_id: RENodeId,
        module_id: NodeModuleId,
        offset: &SubstateOffset,
    ) -> Result<SubstateRefMut, CallFrameError> {
        let node = self
            .nodes
            .get_mut(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))?;

        // TODO: Will clean this up when virtual substates is cleaned up
        match (&node_id, offset) {
            (RENodeId::KeyValueStore(..), SubstateOffset::KeyValueStore(..)) => {
                let entry = node.substates.entry((module_id, offset.clone())).or_insert(
                    RuntimeSubstate::KeyValueStoreEntry(KeyValueStoreEntrySubstate::None),
                );
                Ok(entry.to_ref_mut())
            }
            (RENodeId::NonFungibleStore(..), SubstateOffset::NonFungibleStore(..)) => {
                let entry = node
                    .substates
                    .entry((module_id, offset.clone()))
                    .or_insert(RuntimeSubstate::NonFungible(NonFungibleSubstate(None)));
                Ok(entry.to_ref_mut())
            }
            _ => node
                .substates
                .get_mut(&(module_id, offset.clone()))
                .map(|s| s.to_ref_mut())
                .ok_or(CallFrameError::OffsetDoesNotExist(node_id, offset.clone())),
        }
    }

    pub fn create_node(&mut self, node_id: RENodeId, node: HeapRENode) {
        self.nodes.insert(node_id, node);
    }

    pub fn move_nodes_to_store(
        &mut self,
        track: &mut Track,
        nodes: HashSet<RENodeId>,
    ) -> Result<(), CallFrameError> {
        for node_id in nodes {
            self.move_node_to_store(track, node_id)?;
        }

        Ok(())
    }

    pub fn move_node_to_store(
        &mut self,
        track: &mut Track,
        node_id: RENodeId,
    ) -> Result<(), CallFrameError> {
        let node = self
            .nodes
            .remove(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))?;
        for ((module_id, offset), substate) in node.substates {
            let (_, owned_nodes) = substate.to_ref().references_and_owned_nodes();
            self.move_nodes_to_store(track, owned_nodes)?;
            track.insert_substate(SubstateId(node_id, module_id, offset), substate);
        }

        Ok(())
    }

    pub fn remove_node(&mut self, node_id: RENodeId) -> Result<HeapRENode, CallFrameError> {
        self.nodes
            .remove(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))
    }
}

#[derive(Debug)]
pub struct HeapRENode {
    pub substates: BTreeMap<(NodeModuleId, SubstateOffset), RuntimeSubstate>,
}

impl Into<BucketSubstate> for HeapRENode {
    fn into(mut self) -> BucketSubstate {
        self.substates
            .remove(&(
                NodeModuleId::SELF,
                SubstateOffset::Bucket(BucketOffset::Bucket),
            ))
            .unwrap()
            .into()
    }
}

impl Into<ProofSubstate> for HeapRENode {
    fn into(mut self) -> ProofSubstate {
        self.substates
            .remove(&(
                NodeModuleId::SELF,
                SubstateOffset::Proof(ProofOffset::Proof),
            ))
            .unwrap()
            .into()
    }
}

impl Into<FeeReserveSubstate> for HeapRENode {
    fn into(mut self) -> FeeReserveSubstate {
        self.substates
            .remove(&(
                NodeModuleId::SELF,
                SubstateOffset::FeeReserve(FeeReserveOffset::FeeReserve),
            ))
            .unwrap()
            .into()
    }
}

impl Into<LoggerSubstate> for HeapRENode {
    fn into(mut self) -> LoggerSubstate {
        self.substates
            .remove(&(
                NodeModuleId::SELF,
                SubstateOffset::Logger(LoggerOffset::Logger),
            ))
            .unwrap()
            .into()
    }
}

impl Into<AuthZoneStackSubstate> for HeapRENode {
    fn into(mut self) -> AuthZoneStackSubstate {
        self.substates
            .remove(&(
                NodeModuleId::SELF,
                SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            ))
            .unwrap()
            .into()
    }
}

impl Into<TransactionRuntimeSubstate> for HeapRENode {
    fn into(mut self) -> TransactionRuntimeSubstate {
        self.substates
            .remove(&(
                NodeModuleId::SELF,
                SubstateOffset::TransactionRuntime(TransactionRuntimeOffset::TransactionRuntime),
            ))
            .unwrap()
            .into()
    }
}
