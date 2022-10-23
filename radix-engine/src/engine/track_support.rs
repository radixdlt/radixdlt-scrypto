use indexmap::IndexMap;

use crate::ledger::*;
use crate::model::*;
use crate::state_manager::StateDiff;
use crate::types::*;

/// Keeps track of state changes that that are non-reversible, such as fee payments
pub struct StateTrack<'s> {
    /// The parent state track
    substate_store: &'s dyn ReadableSubstateStore,
    /// Substates either created during the transaction or loaded from substate store
    ///
    /// TODO: can we use Substate instead of `Vec<u8>`?
    /// We're currently blocked by some Substate using `Rc<RefCell<T>>`, which may break
    /// the separation between app state track and base stack track.
    ///
    substates: IndexMap<SubstateId, Option<Vec<u8>>>,
}

impl<'s> StateTrack<'s> {
    pub fn new(substate_store: &'s dyn ReadableSubstateStore) -> Self {
        Self {
            substate_store,
            substates: IndexMap::new(),
        }
    }

    pub fn put_substate(&mut self, substate_id: SubstateId, substate: PersistedSubstate) {
        self.substates
            .insert(substate_id, Some(scrypto_encode(&substate)));
    }

    /// Returns a copy of the substate associated with the given address, if exists
    pub fn get_substate(&mut self, substate_id: &SubstateId) -> Option<PersistedSubstate> {
        // First, try to copy it from the base track
        self.substates
            .get(substate_id)
            .cloned()
            .unwrap_or_else(|| {
                // If not found, load from the substate store
                self.substate_store
                    .get_substate(substate_id)
                    .map(|s| scrypto_encode(&s.substate))
            })
            .map(|x| {
                scrypto_decode(&x).expect(&format!("Failed to decode substate {:?}", substate_id))
            })
    }

    fn get_substate_output_id(
        substate_store: &&'s dyn ReadableSubstateStore,
        substate_id: &SubstateId,
    ) -> Option<OutputId> {
        substate_store.get_substate(&substate_id).map(|s| OutputId {
            substate_id: substate_id.clone(),
            substate_hash: hash(scrypto_encode(&s.substate)),
            version: s.version,
        })
    }

    pub fn generate_diff(&self) -> StateDiff {
        let mut diff = StateDiff::new();

        for (substate_id, substate) in &self.substates {
            if let Some(substate) = substate {
                let next_version = if let Some(existing_output_id) =
                    Self::get_substate_output_id(&self.substate_store, &substate_id)
                {
                    let next_version = existing_output_id.version + 1;
                    diff.down_substates.push(existing_output_id);
                    next_version
                } else {
                    0
                };
                let output_value = OutputValue {
                    substate: scrypto_decode(&substate)
                        .expect(&format!("Failed to decode substate {:?}", substate_id)),
                    version: next_version,
                };
                diff.up_substates.insert(substate_id.clone(), output_value);
            } else {
                // FIXME: How is this being recorded, considering that we're not rejecting the transaction
                // if it attempts to touch some non-existing global addresses?
            }
        }

        diff
    }
}
