use crate::ledger::*;
use crate::state_manager::CommitReceipt;
use crate::types::*;

#[derive(Debug, Clone, Hash, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct VirtualSubstateId(pub SubstateId, pub Vec<u8>);

#[derive(Debug, TypeId, Encode, Decode)]
pub struct StateDiff {
    pub down_virtual_substates: Vec<VirtualSubstateId>,
    pub up_substates: BTreeMap<SubstateId, OutputValue>,
    pub down_substates: Vec<OutputId>,
    pub new_roots: Vec<SubstateId>,
}

impl StateDiff {
    pub fn new() -> Self {
        Self {
            up_substates: BTreeMap::new(),
            down_virtual_substates: Vec::new(),
            down_substates: Vec::new(),
            new_roots: Vec::new(),
        }
    }

    /// Applies the state changes to some substate store.
    pub fn commit<S: WriteableSubstateStore>(&self, store: &mut S) -> CommitReceipt {
        let mut receipt = CommitReceipt::new();

        for virtual_substate_id in &self.down_virtual_substates {
            receipt.virtual_down(virtual_substate_id.clone());
        }

        for output_id in &self.down_substates {
            receipt.down(output_id.clone());
        }
        for (substate_id, output_value) in &self.up_substates {
            let output_id = OutputId {
                substate_id: substate_id.clone(),
                substate_hash: hash(scrypto_encode(&output_value.substate)),
                version: output_value.version,
            };
            receipt.up(output_id);
            store.put_substate(substate_id.clone(), output_value.clone());
        }

        for substate_id in &self.new_roots {
            store.set_root(substate_id.clone());
        }

        receipt
    }
}
