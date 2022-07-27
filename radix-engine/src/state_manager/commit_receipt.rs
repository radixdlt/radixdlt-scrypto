use sbor::rust::collections::*;
use sbor::rust::vec::Vec;

use crate::ledger::*;
use crate::state_manager::VirtualSubstateId;

pub struct CommitReceipt {
    pub virtual_down_substates: HashSet<VirtualSubstateId>,
    pub virtual_up_substates: Vec<OutputId>,
    pub down_substates: HashSet<OutputId>,
    pub up_substates: Vec<OutputId>,
}

impl CommitReceipt {
    pub fn new() -> Self {
        CommitReceipt {
            virtual_down_substates: HashSet::new(),
            down_substates: HashSet::new(),
            virtual_up_substates: Vec::new(),
            up_substates: Vec::new(),
        }
    }

    pub fn virtual_down(&mut self, id: VirtualSubstateId) {
        self.virtual_down_substates.insert(id);
    }

    pub fn virtual_space_up(&mut self, id: OutputId) {
        self.up_substates.push(id);
    }

    pub fn down(&mut self, id: OutputId) {
        self.down_substates.insert(id);
    }

    pub fn up(&mut self, id: OutputId) {
        self.up_substates.push(id);
    }
}
