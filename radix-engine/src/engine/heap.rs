use crate::engine::{CallFrameError, Track};
use crate::fee::FeeReserve;
use crate::model::{
    BucketSubstate, KeyValueStoreEntrySubstate, NonFungibleSubstate, ProofSubstate,
    RawSubstateRefMut, RuntimeSubstate, SubstateRef,
};
use crate::types::{HashMap, HashSet};
use scrypto::engine::types::{BucketOffset, ProofOffset, RENodeId, SubstateId, SubstateOffset};

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
        offset: &SubstateOffset,
    ) -> Result<SubstateRef, CallFrameError> {
        let node = self
            .nodes
            .get_mut(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))?;

        // TODO: Will clean this up when virtual substates is cleaned up
        match (&node_id, offset) {
            (RENodeId::KeyValueStore(..), SubstateOffset::KeyValueStore(..)) => {
                let entry = node.substates.entry(offset.clone()).or_insert(
                    RuntimeSubstate::KeyValueStoreEntry(KeyValueStoreEntrySubstate(None)),
                );
                Ok(entry.to_ref())
            }
            (RENodeId::NonFungibleStore(..), SubstateOffset::NonFungibleStore(..)) => {
                let entry = node
                    .substates
                    .entry(offset.clone())
                    .or_insert(RuntimeSubstate::NonFungible(NonFungibleSubstate(None)));
                Ok(entry.to_ref())
            }
            _ => node
                .substates
                .get(offset)
                .map(|s| s.to_ref())
                .ok_or(CallFrameError::OffsetDoesNotExist(node_id, offset.clone())),
        }
    }

    pub fn get_substate_mut(
        &mut self,
        node_id: RENodeId,
        offset: &SubstateOffset,
    ) -> Result<RawSubstateRefMut, CallFrameError> {
        let node = self
            .nodes
            .get_mut(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))?;

        // TODO: Will clean this up when virtual substates is cleaned up
        match (&node_id, offset) {
            (RENodeId::KeyValueStore(..), SubstateOffset::KeyValueStore(..)) => {
                let entry = node.substates.entry(offset.clone()).or_insert(
                    RuntimeSubstate::KeyValueStoreEntry(KeyValueStoreEntrySubstate(None)),
                );
                Ok(entry.to_ref_mut())
            }
            (RENodeId::NonFungibleStore(..), SubstateOffset::NonFungibleStore(..)) => {
                let entry = node
                    .substates
                    .entry(offset.clone())
                    .or_insert(RuntimeSubstate::NonFungible(NonFungibleSubstate(None)));
                Ok(entry.to_ref_mut())
            }
            _ => node
                .substates
                .get_mut(offset)
                .map(|s| s.to_ref_mut())
                .ok_or(CallFrameError::OffsetDoesNotExist(node_id, offset.clone())),
        }
    }

    pub fn create_node(&mut self, node_id: RENodeId, node: HeapRENode) {
        self.nodes.insert(node_id, node);
    }

    pub fn move_nodes_to_store<R: FeeReserve>(
        &mut self,
        track: &mut Track<R>,
        nodes: HashSet<RENodeId>,
    ) -> Result<(), CallFrameError> {
        for node_id in nodes {
            self.move_node_to_store(track, node_id)?;
        }

        Ok(())
    }

    pub fn move_node_to_store<R: FeeReserve>(
        &mut self,
        track: &mut Track<R>,
        node_id: RENodeId,
    ) -> Result<(), CallFrameError> {
        let node = self
            .nodes
            .remove(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))?;
        for (offset, substate) in node.substates {
            let (_, owned_nodes) = substate.to_ref().references_and_owned_nodes();
            self.move_nodes_to_store(track, owned_nodes)?;
            track.insert_substate(SubstateId(node_id, offset), substate);
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
    pub substates: HashMap<SubstateOffset, RuntimeSubstate>,
}

impl Into<BucketSubstate> for HeapRENode {
    fn into(mut self) -> BucketSubstate {
        self.substates
            .remove(&SubstateOffset::Bucket(BucketOffset::Bucket))
            .unwrap()
            .into()
    }
}

impl Into<ProofSubstate> for HeapRENode {
    fn into(mut self) -> ProofSubstate {
        self.substates
            .remove(&SubstateOffset::Proof(ProofOffset::Proof))
            .unwrap()
            .into()
    }
}
