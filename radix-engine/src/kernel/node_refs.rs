use crate::kernel::substate_io::SubstateDevice;
use radix_engine_common::types::NodeId;
use sbor::rust::ops::{AddAssign, SubAssign};
use utils::prelude::NonIterMap;

pub struct NonGlobalNodeRefs {
    node_refs: NonIterMap<NodeId, (SubstateDevice, usize)>,
}

impl NonGlobalNodeRefs {
    pub fn new() -> Self {
        Self {
            node_refs: NonIterMap::new(),
        }
    }

    pub fn get_ref_device(&self, node_id: &NodeId) -> SubstateDevice {
        let (device, ref_count) = self.node_refs.get(node_id).unwrap();

        if ref_count.eq(&0) {
            panic!("Reference no longer exists");
        }

        *device
    }

    pub fn increment_ref_count(&mut self, node_id: NodeId, device: SubstateDevice) {
        let (_, ref_count) = self.node_refs.entry(node_id).or_insert((device, 0));
        ref_count.add_assign(1);
    }

    pub fn decrement_ref_count(&mut self, node_id: &NodeId) {
        let (_, ref_count) = self
            .node_refs
            .get_mut(node_id)
            .unwrap_or_else(|| panic!("Node {:?} not found", node_id));
        ref_count.sub_assign(1);
    }

    pub fn node_is_referenced(&self, node_id: &NodeId) -> bool {
        self.node_refs
            .get(node_id)
            .map(|(_, ref_count)| ref_count.gt(&0))
            .unwrap_or(false)
    }
}
