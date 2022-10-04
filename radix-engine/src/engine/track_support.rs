use core::ops::RangeFull;

use indexmap::IndexMap;

use crate::ledger::*;
use crate::model::*;
use crate::state_manager::StateDiff;
use crate::state_manager::VirtualSubstateId;
use crate::types::*;

/// Keeps track of state changes that that are non-reversible, such as fee payments
pub struct BaseStateTrack<'s> {
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

impl<'s> BaseStateTrack<'s> {
    pub fn new(substate_store: &'s dyn ReadableSubstateStore) -> Self {
        Self {
            substate_store,
            substates: IndexMap::new(),
        }
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
                match &substate_id {
                    SubstateId(
                        RENodeId::ResourceManager(resource_address),
                        SubstateOffset::Resource(ResourceManagerOffset::NonFungible(key)),
                    ) => {
                        let next_version = if let Some(existing_output_id) =
                            Self::get_substate_output_id(&self.substate_store, &substate_id)
                        {
                            let next_version = existing_output_id.version + 1;
                            diff.down_substates.push(existing_output_id);
                            next_version
                        } else {
                            let parent_address = SubstateId(
                                RENodeId::ResourceManager(*resource_address),
                                SubstateOffset::Resource(ResourceManagerOffset::NonFungibleSpace),
                            );
                            let virtual_output_id =
                                VirtualSubstateId(parent_address, key.0.clone());
                            diff.down_virtual_substates.push(virtual_output_id);
                            0
                        };

                        let output_value = OutputValue {
                            substate: scrypto_decode(&substate)
                                .expect("Failed to decode NonFungibleSubstate"),
                            version: next_version,
                        };
                        diff.up_substates.insert(substate_id.clone(), output_value);
                    }
                    SubstateId(
                        RENodeId::KeyValueStore(kv_store_id),
                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key)),
                    ) => {
                        let next_version = if let Some(existing_output_id) =
                            Self::get_substate_output_id(&self.substate_store, &substate_id)
                        {
                            let next_version = existing_output_id.version + 1;
                            diff.down_substates.push(existing_output_id);
                            next_version
                        } else {
                            let parent_address = SubstateId(
                                RENodeId::KeyValueStore(*kv_store_id),
                                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Space),
                            );
                            let virtual_output_id = VirtualSubstateId(parent_address, key.clone());
                            diff.down_virtual_substates.push(virtual_output_id);
                            0
                        };

                        let output_value = OutputValue {
                            substate: scrypto_decode(&substate)
                                .expect("Failed to decode KeyValueStoreEntrySubstate"),
                            version: next_version,
                        };
                        diff.up_substates.insert(substate_id.clone(), output_value);
                    }
                    _ => {
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
                    }
                }
            } else {
                // FIXME: How is this being recorded, considering that we're not rejecting the transaction
                // if it attempts to touch some non-existing global addresses?
            }
        }

        diff
    }
}

#[derive(Debug)]
pub enum StateTrackError {
    RENodeAlreadyTouched,
}

/// Keeps track of state changes that may be rolled back according to transaction status
pub struct AppStateTrack<'s> {
    /// The parent state track
    base_state_track: BaseStateTrack<'s>,
    /// Substates either created during the transaction or loaded from the base state track
    substates: IndexMap<SubstateId, Option<Vec<u8>>>,
}

impl<'s> AppStateTrack<'s> {
    pub fn new(base_state_track: BaseStateTrack<'s>) -> Self {
        Self {
            base_state_track,
            substates: IndexMap::new(),
        }
    }

    /// Returns a copy of the substate associated with the given address, if exists
    pub fn get_substate(&mut self, substate_id: &SubstateId) -> Option<Substate> {
        self.substates
            .entry(substate_id.clone())
            .or_insert_with(|| {
                // First, try to copy it from the base track
                self.base_state_track
                    .substates
                    .get(substate_id)
                    .cloned()
                    .unwrap_or_else(|| {
                        // If not found, load from the substate store
                        self.base_state_track
                            .substate_store
                            .get_substate(substate_id)
                            .map(|s| scrypto_encode(&s.substate))
                    })
            })
            .as_ref()
            .map(|x| {
                scrypto_decode(x).expect(&format!("Failed to decode substate {:?}", substate_id))
            })
    }

    /// Creates a new substate and updates an existing one
    pub fn put_substate(&mut self, substate_id: SubstateId, substate: Substate) {
        self.substates
            .insert(substate_id, Some(scrypto_encode(&substate)));
    }

    /// Returns a copy of the substate associated with the given address from the base track
    pub fn get_substate_from_base(&mut self, substate_id: &SubstateId) -> Option<Substate> {
        self.base_state_track
            .substates
            .entry(substate_id.clone())
            .or_insert_with(|| {
                // Load from the substate store
                self.base_state_track
                    .substate_store
                    .get_substate(substate_id)
                    .map(|s| scrypto_encode(&s.substate))
            })
            .as_ref()
            .map(|x| {
                scrypto_decode(x).expect(&format!("Failed to decode substate {:?}", substate_id))
            })
    }

    /// Creates a new substate and updates an existing one to the base track
    pub fn put_substate_to_base(&mut self, substate_id: SubstateId, substate: Substate) {
        assert!(!self.substates.contains_key(&substate_id));

        self.base_state_track
            .substates
            .insert(substate_id, Some(scrypto_encode(&substate)));
    }

    /// Commit all state changes into base state track
    pub fn commit(&mut self) {
        self.base_state_track
            .substates
            .extend(self.substates.drain(RangeFull));
    }

    /// Rollback all state changes
    pub fn rollback(&mut self) {
        self.substates.clear();
    }

    /// Unwraps into the base state track
    pub fn into_base(self) -> BaseStateTrack<'s> {
        self.base_state_track
    }
}
