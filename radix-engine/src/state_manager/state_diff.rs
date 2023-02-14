use crate::ledger::*;
use crate::state_manager::CommitReceipt;
use crate::types::*;
use radix_engine_interface::api::types::SubstateId;
use radix_engine_interface::crypto::hash;

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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
                substate_hash: hash(
                    scrypto_encode(&output_value.substate).unwrap_or_else(|err| {
                        panic!(
                            "Could not encode newly-committed substate: {:?}. Substate: {:?}",
                            err, &output_value.substate
                        )
                    }),
                ),
                version: output_value.version,
            };
            receipt.up(output_id);
            store.put_substate(substate_id.clone(), output_value.clone());
        }

        receipt
    }

    pub fn up_substate_ids(&self) -> BTreeSet<&SubstateId> {
        self.up_substates.iter().map(|(id, _)| id).collect()
    }

    pub fn down_substate_ids(&self) -> BTreeSet<&SubstateId> {
        self.down_substates
            .iter()
            .map(|output| &output.substate_id)
            .collect()
    }

    pub fn up_substate_offsets(&self) -> BTreeMap<&SubstateOffset, usize> {
        let mut counter = BTreeMap::new();
        for s in &self.up_substates {
            *counter.entry(&s.0 .1).or_default() += 1;
        }
        counter
    }

    pub fn down_substate_offsets(&self) -> BTreeMap<&SubstateOffset, usize> {
        let mut counter = BTreeMap::new();
        for s in &self.down_substates {
            *counter.entry(&s.substate_id.1).or_default() += 1;
        }
        counter
    }
}
