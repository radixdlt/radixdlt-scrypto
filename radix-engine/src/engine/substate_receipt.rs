use crate::engine::track::VirtualSubstateId;
use crate::engine::SubstateParentId;
use sbor::rust::collections::*;
use sbor::rust::ops::RangeFull;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::hash;

use crate::ledger::*;

use super::Address;

pub struct CommitReceipt {
    pub virtual_down_substates: HashSet<HardVirtualSubstateId>,
    pub down_substates: HashSet<PhysicalSubstateId>,
    pub virtual_up_substates: Vec<PhysicalSubstateId>,
    pub up_substates: Vec<PhysicalSubstateId>,
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

    fn down(&mut self, id: PhysicalSubstateId) {
        self.down_substates.insert(id);
    }

    fn virtual_space_up(&mut self, id: PhysicalSubstateId) {
        self.up_substates.push(id);
    }

    fn up(&mut self, id: PhysicalSubstateId) {
        self.up_substates.push(id);
    }
}

#[derive(Debug, Clone, Hash, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct HardVirtualSubstateId(PhysicalSubstateId, Vec<u8>);

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum SubstateOperation {
    VirtualDown(VirtualSubstateId),
    Down(PhysicalSubstateId),
    VirtualUp(Address),
    Up(Address, Vec<u8>),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct SubstateOperationsReceipt {
    pub substate_operations: Vec<SubstateOperation>,
}

impl SubstateOperationsReceipt {
    /// Commits changes to the underlying substate store.
    pub fn commit<S: WriteableSubstateStore>(mut self, store: &mut S) -> CommitReceipt {
        let hash = hash(scrypto_encode(&self));
        let mut receipt = CommitReceipt::new();
        let mut id_gen = SubstateIdGenerator::new(hash);

        for instruction in self.substate_operations.drain(RangeFull) {
            match instruction {
                SubstateOperation::VirtualDown(VirtualSubstateId(parent_id, key)) => {
                    let parent_hard_id = match parent_id {
                        SubstateParentId::Exists(real_id) => real_id,
                        SubstateParentId::New(index) => {
                            PhysicalSubstateId(hash, index.try_into().unwrap())
                        }
                    };
                    let virtual_substate_id = HardVirtualSubstateId(parent_hard_id, key);
                    receipt.virtual_down(virtual_substate_id);
                }
                SubstateOperation::Down(substate_id) => receipt.down(substate_id),
                SubstateOperation::VirtualUp(address) => {
                    let phys_id = id_gen.next();
                    receipt.virtual_space_up(phys_id.clone());
                    store.put_space(address, phys_id);
                }
                SubstateOperation::Up(address, value) => {
                    let phys_id = id_gen.next();
                    receipt.up(phys_id.clone());
                    let substate = Substate { value, phys_id };
                    store.put_substate(address, substate);
                }
            }
        }

        receipt
    }
}
