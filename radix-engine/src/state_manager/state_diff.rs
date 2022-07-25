use sbor::rust::collections::*;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::hash;

use crate::engine::Address;
use crate::engine::*;
use crate::ledger::*;
use crate::state_manager::CommitReceipt;
use crate::state_manager::HardVirtualSubstateId;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum SubstateParentId {
    Exists(OutputId),
    New(Address),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct VirtualSubstateId(pub SubstateParentId, pub Vec<u8>);

#[derive(Debug, TypeId, Encode, Decode)]
pub struct StateDiff {
    pub up_virtual_substates: BTreeSet<Address>,
    pub up_substates: BTreeMap<Address, Substate>,
    pub down_virtual_substates: Vec<VirtualSubstateId>,
    pub down_substates: Vec<OutputId>,
}

impl StateDiff {
    pub fn new() -> Self {
        Self {
            up_virtual_substates: BTreeSet::new(),
            up_substates: BTreeMap::new(),
            down_virtual_substates: Vec::new(),
            down_substates: Vec::new(),
        }
    }

    /// Applies the state changes to some substate store.
    pub fn commit<S: WriteableSubstateStore>(&self, store: &mut S) -> CommitReceipt {
        let hash = hash(scrypto_encode(self));
        let mut receipt = CommitReceipt::new();
        let mut id_gen = OutputIdGenerator::new(hash);
        let mut virtual_outputs = HashMap::new();

        for space_address in &self.up_virtual_substates {
            let output_id = id_gen.next();
            receipt.virtual_space_up(output_id.clone());
            store.put_space(space_address.clone(), output_id.clone());
            virtual_outputs.insert(space_address, output_id);
        }
        for output_id in &self.down_substates {
            receipt.down(output_id.clone());
        }
        for VirtualSubstateId(parent_id, key) in &self.down_virtual_substates {
            let parent_hard_id = match parent_id {
                SubstateParentId::Exists(real_id) => real_id.clone(),
                SubstateParentId::New(key) => virtual_outputs.get(&key).cloned().unwrap(),
            };
            let virtual_substate_id = HardVirtualSubstateId(parent_hard_id, key.clone());
            receipt.virtual_down(virtual_substate_id);
        }
        for (address, value) in &self.up_substates {
            let output_id = id_gen.next();
            receipt.up(output_id.clone());
            store.put_substate(
                address.clone(),
                Output {
                    substate: value.clone(),
                    output_id: output_id,
                },
            );
        }
        todo!()
    }
}
