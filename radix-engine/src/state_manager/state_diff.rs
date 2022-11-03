use crate::ledger::*;
use crate::state_manager::CommitReceipt;
use crate::types::*;

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct StateDiff {
    pub up_substates: BTreeMap<SubstateId, OutputValue>,
    pub down_substates: Vec<OutputId>,
}

impl StateDiff {
    pub fn new() -> Self {
        Self {
            up_substates: BTreeMap::new(),
            down_substates: Vec::new(),
        }
    }

    /// Applies the state changes to some substate store.
    pub fn commit<S: WriteableSubstateStore>(&self, store: &mut S) -> CommitReceipt {
        let mut receipt = CommitReceipt::new();

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

        receipt
    }
}
