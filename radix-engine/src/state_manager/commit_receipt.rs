use crate::types::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, ScryptoSbor)]
pub struct OutputId {
    pub substate_id: Vec<u8>,
    pub substate_hash: Hash,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct OutputValue {
    pub substate_value: Vec<u8>,
    pub version: u32,
}

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
