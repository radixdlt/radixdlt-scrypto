use crate::ledger::*;
use crate::types::*;

pub struct CommitReceipt {
    pub inputs: Vec<OutputId>,
    pub outputs: Vec<OutputId>,
}

impl CommitReceipt {
    pub fn new() -> Self {
        CommitReceipt {
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn down(&mut self, id: OutputId) {
        self.inputs.push(id);
    }

    pub fn up(&mut self, id: OutputId) {
        self.outputs.push(id);
    }
}
