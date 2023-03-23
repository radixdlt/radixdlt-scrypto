use super::*;
use crate::types::*;

#[derive(Debug, Clone, ScryptoSbor)]
pub struct StateDiff {
    pub up_substates: BTreeMap<SubstateId, OutputValue>,
    pub down_substates: BTreeSet<OutputId>,
}

impl StateDiff {
    pub fn new() -> Self {
        Self {
            up_substates: BTreeMap::new(),
            down_substates: BTreeSet::new(),
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
}
