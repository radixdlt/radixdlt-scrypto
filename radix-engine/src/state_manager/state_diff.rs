use crate::ledger::*;
use crate::state_manager::CommitReceipt;
use crate::system::node_substates::PersistedSubstate;
use crate::types::*;
use radix_engine_interface::crypto::hash;

#[derive(Debug, Clone, ScryptoSbor)]
pub enum IterableSubstateDiff {
    Insert(PersistedSubstate),
    Remove,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub enum IterableNodeDiff {
    New(BTreeMap<SubstateOffset, PersistedSubstate>),
    Update(BTreeMap<SubstateOffset, IterableSubstateDiff>),
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct StateDiff {
    pub up_substates: BTreeMap<SubstateId, OutputValue>,
    pub down_substates: BTreeSet<OutputId>,

    pub iterable_nodes_update: BTreeMap<(RENodeId, NodeModuleId), IterableNodeDiff>,
}

impl StateDiff {
    pub fn new() -> Self {
        Self {
            up_substates: BTreeMap::new(),
            down_substates: BTreeSet::new(),
            iterable_nodes_update: BTreeMap::new(),
        }
    }

    /// Merges the changes from the given other diff into this one in a naive way, by simply
    /// extending the tracked up/down substate collections (i.e. without resolving any potential
    /// up->down or down->up interactions coming from the other diff).
    pub fn extend(&mut self, other: StateDiff) {
        let mut other = other;
        self.up_substates.extend(other.up_substates);
        self.down_substates.append(&mut other.down_substates);
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

        for (node_module, node_diff) in &self.iterable_nodes_update {
            match node_diff {
                IterableNodeDiff::New(substates) => {
                    for (offset, substate) in substates {
                        let substate_id = SubstateId(
                            node_module.0.clone(),
                            node_module.1.clone(),
                            offset.clone(),
                        );
                        store.put_substate(
                            substate_id,
                            OutputValue {
                                substate: substate.clone(),
                                version: 0u32, // TODO: Remove
                            },
                        );
                    }
                }
                IterableNodeDiff::Update(updates) => {
                    for (offset, substate_diff) in updates {
                        let substate_id = SubstateId(
                            node_module.0.clone(),
                            node_module.1.clone(),
                            offset.clone(),
                        );
                        match substate_diff {
                            IterableSubstateDiff::Insert(substate) => {
                                store.put_substate(
                                    substate_id,
                                    OutputValue {
                                        substate: substate.clone(),
                                        version: 0u32, // TODO: Remove
                                    },
                                );
                            }
                            IterableSubstateDiff::Remove => {
                                store.remove_substate(&substate_id);
                            }
                        }
                    }
                }
            }
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
            *counter.entry(&s.0 .2).or_default() += 1;
        }
        counter
    }

    pub fn down_substate_offsets(&self) -> BTreeMap<&SubstateOffset, usize> {
        let mut counter = BTreeMap::new();
        for s in &self.down_substates {
            *counter.entry(&s.substate_id.2).or_default() += 1;
        }
        counter
    }
}
