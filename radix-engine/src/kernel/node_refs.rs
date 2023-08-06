use std::ops::{AddAssign, SubAssign};
use radix_engine_common::types::NodeId;
use utils::prelude::NonIterMap;

pub struct NodeRefs {
    node_refs: NonIterMap<NodeId, usize>,
}

impl NodeRefs {
    pub fn new() -> Self {
        Self {
            node_refs: NonIterMap::new(),
        }
    }

    pub fn add_borrow(&mut self, node_id: &NodeId) {
        self.node_refs.entry(*node_id)
            .or_insert(0)
            .add_assign(1);
    }

    pub fn release_borrow(&mut self, node_id: &NodeId) {
        self.node_refs
            .get_mut(node_id)
            .unwrap_or_else(|| panic!("Node {:?} not found", node_id))
            .sub_assign(1);
    }

    pub fn node_is_referenced(&self, node_id: &NodeId) -> bool {
        self.node_refs.get(node_id).map(|count| count.gt(&0)).unwrap_or(false)
    }
}