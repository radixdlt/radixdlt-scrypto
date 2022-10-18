use crate::engine::{CallFrameError, HeapRootRENode};
use crate::types::{HashMap, HashSet};
use scrypto::engine::types::RENodeId;

pub struct Heap {
    pub nodes: HashMap<RENodeId, HeapRootRENode>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn get_node_mut(
        &mut self,
        node_id: RENodeId,
    ) -> Result<&mut HeapRootRENode, CallFrameError> {
        self.nodes
            .get_mut(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))
    }

    pub fn get_node(&self, node_id: RENodeId) -> Result<&HeapRootRENode, CallFrameError> {
        self.nodes
            .get(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))
    }

    pub fn create_node(&mut self, node_id: RENodeId, node: HeapRootRENode) {
        self.nodes.insert(node_id, node);
    }

    pub fn move_nodes_to_node(
        &mut self,
        nodes: HashSet<RENodeId>,
        to: RENodeId,
    ) -> Result<(), CallFrameError> {
        let mut child_nodes = HashMap::new();

        for child_id in nodes {
            let node = self.remove_node(child_id)?;
            child_nodes.extend(node.to_nodes(child_id));
        }

        self.get_node_mut(to)?.child_nodes.extend(child_nodes);

        Ok(())
    }

    pub fn remove_node(&mut self, node_id: RENodeId) -> Result<HeapRootRENode, CallFrameError> {
        self.nodes
            .remove(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))
    }
}
