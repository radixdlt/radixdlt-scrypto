use radix_engine_common::types::NodeId;
use std::ops::{AddAssign, SubAssign};
use utils::prelude::NonIterMap;

pub struct NonGlobalNodeRefs {
    node_refs: NonIterMap<NodeId, usize>,
}

impl NonGlobalNodeRefs {
    pub fn new() -> Self {
        Self {
            node_refs: NonIterMap::new(),
        }
    }

    pub fn increment_ref_count(&mut self, node_id: &NodeId) {
        self.node_refs.entry(*node_id).or_insert(0).add_assign(1);
    }

    pub fn decrement_ref_count(&mut self, node_id: &NodeId) {
        self.node_refs
            .get_mut(node_id)
            .unwrap_or_else(|| panic!("Node {:?} not found", node_id))
            .sub_assign(1);
    }

    pub fn node_is_referenced(&self, node_id: &NodeId) -> bool {
        self.node_refs
            .get(node_id)
            .map(|count| count.gt(&0))
            .unwrap_or(false)
    }
}
