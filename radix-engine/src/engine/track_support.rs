use core::ops::RangeFull;

use indexmap::{IndexMap, IndexSet};
use scrypto::crypto::hash;
use scrypto::prelude::scrypto_encode;

use crate::engine::*;
use crate::ledger::*;
use crate::state_manager::StateDiff;
use crate::state_manager::VirtualSubstateId;

/// Keeps track of state changes that that are non-reversible, such as fee payments
pub struct BaseStateTrack<'s> {
    /// The parent state track
    substate_store: &'s dyn ReadableSubstateStore,
    /// Substates either created during the transaction or loaded from substate store
    substates: IndexMap<Address, Option<Substate>>,
    /// Spaces created during the transaction
    spaces: IndexSet<Address>,
}

/// Keeps track of state changes that may be rolled back according to transaction status
pub struct AppStateTrack<'s> {
    /// The parent state track
    base_state_track: BaseStateTrack<'s>,
    /// Substates either created during the transaction or loaded from the base state track
    substates: IndexMap<Address, Option<Substate>>,
    /// Spaces created during the transaction
    spaces: IndexSet<Address>,
}

impl<'s> BaseStateTrack<'s> {
    pub fn new(substate_store: &'s dyn ReadableSubstateStore) -> Self {
        Self {
            substate_store,
            substates: IndexMap::new(),
            spaces: IndexSet::new(),
        }
    }

    fn get_substate_output_id(
        substate_store: &&'s dyn ReadableSubstateStore,
        address: &Address,
    ) -> Option<OutputId> {
        substate_store.get_substate(&address).map(|s| OutputId {
            address: address.clone(),
            substate_hash: hash(scrypto_encode(&s.substate)),
            version: s.version,
        })
    }

    pub fn generate_diff(&self) -> StateDiff {
        let mut diff = StateDiff::new();

        for space in &self.spaces {
            diff.up_virtual_substates.insert(space.clone());
        }

        for (address, substate) in &self.substates {
            if let Some(substate) = substate {
                match &address {
                    Address::NonFungible(resource_address, key) => {
                        let next_version = if let Some(existing_output_id) =
                            Self::get_substate_output_id(&self.substate_store, &address)
                        {
                            let next_version = existing_output_id.version + 1;
                            diff.down_substates.push(existing_output_id);
                            next_version
                        } else {
                            let parent_address = Address::NonFungibleSpace(*resource_address);
                            let virtual_output_id = VirtualSubstateId(parent_address, key.clone());
                            diff.down_virtual_substates.push(virtual_output_id);
                            0
                        };

                        let output_value = OutputValue {
                            substate: substate.clone(),
                            version: next_version,
                        };
                        diff.up_substates.insert(address.clone(), output_value);
                    }
                    Address::KeyValueStoreEntry(kv_store_id, key) => {
                        let next_version = if let Some(existing_output_id) =
                            Self::get_substate_output_id(&self.substate_store, &address)
                        {
                            let next_version = existing_output_id.version + 1;
                            diff.down_substates.push(existing_output_id);
                            next_version
                        } else {
                            let parent_address = Address::KeyValueStoreSpace(*kv_store_id);
                            let virtual_output_id = VirtualSubstateId(parent_address, key.clone());
                            diff.down_virtual_substates.push(virtual_output_id);
                            0
                        };

                        let output_value = OutputValue {
                            substate: substate.clone(),
                            version: next_version,
                        };
                        diff.up_substates.insert(address.clone(), output_value);
                    }
                    _ => {
                        let next_version = if let Some(existing_output_id) =
                            Self::get_substate_output_id(&self.substate_store, &address)
                        {
                            let next_version = existing_output_id.version + 1;
                            diff.down_substates.push(existing_output_id);
                            next_version
                        } else {
                            0
                        };
                        let output_value = OutputValue {
                            substate: substate.clone(),
                            version: next_version,
                        };
                        diff.up_substates.insert(address.clone(), output_value);
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
    ValueAlreadyTouched,
}

impl<'s> AppStateTrack<'s> {
    pub fn new(base_state_track: BaseStateTrack<'s>) -> Self {
        Self {
            base_state_track,
            substates: IndexMap::new(),
            spaces: IndexSet::new(),
        }
    }

    /// Returns a copy of the substate associated with the given address, if exists
    pub fn get_substate(&mut self, address: &Address) -> Option<Substate> {
        self.substates
            .entry(address.clone())
            .or_insert_with(|| {
                // First, try to copy it from the base track
                self.base_state_track
                    .substates
                    .get(address)
                    .cloned()
                    .unwrap_or_else(|| {
                        // If not found, load from the substate store
                        self.base_state_track
                            .substate_store
                            .get_substate(address)
                            .map(|s| s.substate)
                    })
            })
            .clone()
    }

    /// Returns a copy of the substate associated with the given address from the base track
    pub fn get_substate_from_base(
        &mut self,
        address: &Address,
    ) -> Result<Option<Substate>, StateTrackError> {
        if self.substates.contains_key(address) {
            return Err(StateTrackError::ValueAlreadyTouched);
        }

        Ok(self
            .base_state_track
            .substates
            .entry(address.clone())
            .or_insert_with(|| {
                // Load from the substate store
                self.base_state_track
                    .substate_store
                    .get_substate(address)
                    .map(|s| s.substate)
            })
            .clone())
    }

    /// Creates a new substate and updates an existing one
    pub fn put_substate(&mut self, address: Address, substate: Substate) {
        self.substates.insert(address, Some(substate));
    }

    /// Creates a new substate and updates an existing one to the base track
    pub fn put_substate_to_base(&mut self, address: Address, substate: Substate) {
        assert!(!self.substates.contains_key(&address));

        self.base_state_track
            .substates
            .insert(address, Some(substate));
    }

    /// Creates a new space, assuming address does not exist
    pub fn put_space(&mut self, address: Address) {
        self.spaces.insert(address);
    }

    /// Flush all changes to base state track
    pub fn flush(&mut self) {
        self.base_state_track
            .substates
            .extend(self.substates.drain(RangeFull));
        self.base_state_track
            .spaces
            .extend(self.spaces.drain(RangeFull));
    }

    /// Unwraps into the base state track
    pub fn into_base(self) -> BaseStateTrack<'s> {
        self.base_state_track
    }
}
