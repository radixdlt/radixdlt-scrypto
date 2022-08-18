use crate::ledger::*;
use crate::state_manager::VirtualSubstateId;
use crate::types::*;

pub struct CommitReceipt {
    pub virtual_inputs: Vec<VirtualSubstateId>,
    pub inputs: Vec<OutputId>,
    pub outputs: Vec<OutputId>,
}

impl CommitReceipt {
    pub fn new() -> Self {
        CommitReceipt {
            virtual_inputs: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn virtual_down(&mut self, id: VirtualSubstateId) {
        self.virtual_inputs.push(id);
    }

    pub fn down(&mut self, id: OutputId) {
        self.inputs.push(id);
    }

    pub fn up(&mut self, id: OutputId) {
        self.outputs.push(id);
    }
}
