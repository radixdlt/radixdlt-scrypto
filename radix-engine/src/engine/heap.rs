use crate::engine::{CallFrameError, HeapRootRENode};
use crate::types::HashMap;
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
        self.nodes.get_mut(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))
    }

    pub fn get_node(
        &self,
        node_id: RENodeId,
    ) -> Result<&HeapRootRENode, CallFrameError> {
        self.nodes.get(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))
    }

    pub fn create_node(&mut self, node_id: RENodeId, node: HeapRootRENode) {
        self.nodes.insert(node_id, node);
    }

    pub fn remove_node(
        &mut self,
        node_id: RENodeId,
    ) -> Result<HeapRootRENode, CallFrameError> {
        self.nodes.remove(&node_id)
            .ok_or(CallFrameError::RENodeNotOwned(node_id))
    }
}
