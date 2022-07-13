use crate::engine::track::VirtualSubstateId;
use crate::engine::{Substate, SubstateParentId};
use sbor::rust::collections::*;
use sbor::rust::ops::RangeFull;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::hash;

use crate::ledger::*;

pub struct CommitReceipt {
    pub virtual_down_substates: HashSet<HardVirtualSubstateId>,
    pub down_substates: HashSet<OutputId>,
    pub virtual_up_substates: Vec<OutputId>,
    pub up_substates: Vec<OutputId>,
}

impl CommitReceipt {
    fn new() -> Self {
        CommitReceipt {
            virtual_down_substates: HashSet::new(),
            down_substates: HashSet::new(),
            virtual_up_substates: Vec::new(),
            up_substates: Vec::new(),
        }
    }

    fn virtual_down(&mut self, id: HardVirtualSubstateId) {
        self.virtual_down_substates.insert(id);
    }

    fn down(&mut self, id: OutputId) {
        self.down_substates.insert(id);
    }

    fn virtual_space_up(&mut self, id: OutputId) {
        self.up_substates.push(id);
    }

    fn up(&mut self, id: OutputId) {
        self.up_substates.push(id);
    }
}

#[derive(Debug, Clone, Hash, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct HardVirtualSubstateId(OutputId, Vec<u8>);

// TODO: Update encoding scheme here to not take up so much space with the enum strings
#[derive(Debug, TypeId, Encode, Decode)]
pub enum SubstateOperation {
    VirtualDown(VirtualSubstateId),
    Down(OutputId),
    VirtualUp(Vec<u8>),
    Up(Vec<u8>, Substate),
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct SubstateOperationsReceipt {
    pub substate_operations: Vec<SubstateOperation>,
}

impl SubstateOperationsReceipt {
    /// Commits changes to the underlying ledger.
    /// Currently none of these objects are deleted so all commits are puts
    pub fn commit<S: WriteableSubstateStore>(mut self, store: &mut S) -> CommitReceipt {
        let hash = hash(scrypto_encode(&self));
        let mut receipt = CommitReceipt::new();
        let mut id_gen = OutputIdGenerator::new(hash);

        for instruction in self.substate_operations.drain(RangeFull) {
            match instruction {
                SubstateOperation::VirtualDown(VirtualSubstateId(parent_id, key)) => {
                    let parent_hard_id = match parent_id {
                        SubstateParentId::Exists(real_id) => real_id,
                        SubstateParentId::New(index) => OutputId(hash, index.try_into().unwrap()),
                    };
                    let virtual_substate_id = HardVirtualSubstateId(parent_hard_id, key);
                    receipt.virtual_down(virtual_substate_id);
                }
                SubstateOperation::Down(substate_id) => receipt.down(substate_id),
                SubstateOperation::VirtualUp(address) => {
                    let phys_id = id_gen.next();
                    receipt.virtual_space_up(phys_id.clone());
                    store.put_space(&address, phys_id);
                }
                SubstateOperation::Up(key, value) => {
                    let phys_id = id_gen.next();
                    receipt.up(phys_id.clone());
                    let substate = Output { value, phys_id };
                    store.put_substate(&key, substate);
                }
            }
        }

        receipt
    }
}
