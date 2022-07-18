use indexmap::IndexSet;
use sbor::rust::collections::*;
use scrypto::crypto::Hash;
use transaction::validation::IdAllocator;

use crate::engine::Address;
use crate::ledger::*;

use super::{
    track::VirtualSubstateId, SubstateOperation, SubstateOperationsReceipt, SubstateParentId,
};

pub enum StateTrackParent {
    SubstateStore(Box<dyn ReadableSubstateStore>, Hash, IdAllocator),
    StateTrack(Box<StateTrack>),
}

pub struct StateTrack {
    /// The parent state track
    parent: StateTrackParent,
    /// Substates either created by transaction or loaded from substate store
    substates: HashMap<Address, Option<Vec<u8>>>,
    /// Spaces created by transaction
    spaces: IndexSet<Address>,
}

impl StateTrack {
    // TODO: produce substate update receipt

    pub fn new(parent: StateTrackParent) -> Self {
        Self {
            parent,
            substates: HashMap::new(),
            spaces: IndexSet::new(),
        }
    }

    pub fn get_substate(&mut self, address: &Address) -> Option<Vec<u8>> {
        self.substates
            .entry(address.clone())
            .or_insert_with(|| match &mut self.parent {
                StateTrackParent::SubstateStore(store, ..) => {
                    store.get_substate(address).map(|s| s.value)
                }
                StateTrackParent::StateTrack(track) => track.get_substate(address),
            })
            .clone()
    }

    pub fn put_substate(&mut self, address: Address, substate: Vec<u8>) {
        self.substates.insert(address, Some(substate));
    }

    pub fn put_space(&mut self, address: Address) {
        self.spaces.insert(address);
    }

    // Flush all changes to underlying track
    pub fn flush(&mut self) {}

    pub fn into_inner(self) -> Self {
        match self.parent {
            StateTrackParent::SubstateStore(..) => self,
            StateTrackParent::StateTrack(track) => Self::into_inner(*track),
        }
    }

    // TODO: replace recursion with iteration
    
    fn get_substate_id(parent: &StateTrackParent, address: &Address) -> Option<PhysicalSubstateId> {
        match parent {
            StateTrackParent::SubstateStore(store, ..) => {
                store.get_substate(&address).map(|s| s.phys_id)
            }
            StateTrackParent::StateTrack(track) => Self::get_substate_id(&track.parent, address),
        }
    }

    fn get_space_substate_id(
        parent: &StateTrackParent,
        address: &Address,
    ) -> Option<PhysicalSubstateId> {
        match parent {
            StateTrackParent::SubstateStore(store, ..) => store.get_space(&address),
            StateTrackParent::StateTrack(track) => {
                Self::get_space_substate_id(&track.parent, address)
            }
        }
    }

    fn get_substate_parent_id(
        spaces: &IndexSet<Address>,
        parent: &StateTrackParent,
        space_address: &Address,
    ) -> SubstateParentId {
        if let Some(index) = spaces.get_index_of(space_address) {
            SubstateParentId::New(index)
        } else {
            let substate_id = Self::get_space_substate_id(parent, space_address)
                .expect("Attempted to locate non-existing space");
            SubstateParentId::Exists(substate_id)
        }
    }

    pub fn summarize_state_changes(mut self) -> SubstateOperationsReceipt {
        let mut store_instructions = Vec::new();

        // Must be put in front of substate changes to maintain valid parent ids.
        for space in &self.spaces {
            store_instructions.push(SubstateOperation::VirtualUp(space.encode()));
        }

        for (address, substate) in self.substates.drain() {
            if let Some(substate) = substate {
                match &address {
                    Address::NonFungible(resource_address, key) => {
                        let parent_address = Address::Resource(*resource_address);

                        if let Some(existing_substate_id) =
                            Self::get_substate_id(&self.parent, &parent_address)
                        {
                            store_instructions.push(SubstateOperation::Down(existing_substate_id));
                        } else {
                            let parent_id = Self::get_substate_parent_id(
                                &self.spaces,
                                &self.parent,
                                &parent_address,
                            );
                            let virtual_substate_id = VirtualSubstateId(parent_id, key.clone());
                            store_instructions
                                .push(SubstateOperation::VirtualDown(virtual_substate_id));
                        }

                        store_instructions.push(SubstateOperation::Up(address.encode(), substate));
                    }
                    Address::KeyValueStoreEntry(component_id, kv_store_id, key) => {
                        let parent_address = Address::KeyValueStore(*component_id, *kv_store_id);

                        if let Some(existing_substate_id) =
                            Self::get_substate_id(&self.parent, &parent_address)
                        {
                            store_instructions.push(SubstateOperation::Down(existing_substate_id));
                        } else {
                            let parent_id = Self::get_substate_parent_id(
                                &self.spaces,
                                &self.parent,
                                &parent_address,
                            );
                            let virtual_substate_id = VirtualSubstateId(parent_id, key.clone());
                            store_instructions
                                .push(SubstateOperation::VirtualDown(virtual_substate_id));
                        }

                        store_instructions.push(SubstateOperation::Up(address.encode(), substate));
                    }
                    _ => {
                        if let Some(previous_substate_id) =
                            Self::get_substate_id(&self.parent, &address)
                        {
                            store_instructions.push(SubstateOperation::Down(previous_substate_id));
                        }
                        store_instructions.push(SubstateOperation::Up(address.encode(), substate));
                    }
                }
            } else {
                // FIXME: How is this being recorded, considering that we're not rejecting the transaction
                // if it attempts to touch some non-existing global addresses?
            }
        }

        SubstateOperationsReceipt {
            substate_operations: store_instructions,
        }
    }
}
