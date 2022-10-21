use crate::engine::{CallFrameError, HeapRENode, Track};
use crate::fee::FeeReserve;
use crate::model::{
    KeyValueStoreEntrySubstate, NonFungibleSubstate, RawSubstateRefMut, RuntimeSubstate,
    SubstateRef,
};
use crate::types::{HashMap, HashSet};
use scrypto::engine::types::{RENodeId, SubstateId, SubstateOffset};

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

    pub fn get_children(&self, node_id: RENodeId) -> Result<&HashSet<RENodeId>, CallFrameError> {
        self.nodes
            .get(&node_id)
            .map(|n| &n.child_nodes)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))
    }

    pub fn create_node(&mut self, node_id: RENodeId, node: HeapRENode) {
        self.nodes.insert(node_id, node);
    }

    pub fn add_child_nodes(
        &mut self,
        node_ids: HashSet<RENodeId>,
        to: RENodeId,
    ) -> Result<(), CallFrameError> {
        for node_id in &node_ids {
            // Sanity check
            if !self.nodes.contains_key(&node_id) {
                return Err(CallFrameError::RENodeNotOwned(*node_id));
            }
        }

        let heap_node = self
            .nodes
            .get_mut(&to)
            .ok_or(CallFrameError::RENodeNotOwned(to))?;
        heap_node.child_nodes.extend(node_ids);

        Ok(())
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
            track.insert_substate(SubstateId(node_id, offset), substate);
        }

        self.move_nodes_to_store(track, node.child_nodes)?;

        Ok(())
    }

    pub fn remove_node(&mut self, node_id: RENodeId) -> Result<HeapRENode, CallFrameError> {
        self.nodes
            .remove(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))
    }
}
