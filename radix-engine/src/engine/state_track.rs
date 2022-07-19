use indexmap::IndexSet;
use sbor::rust::collections::*;
use sbor::rust::rc::Rc;
use scrypto::buffer::{scrypto_decode, scrypto_encode};

use crate::engine::Address;
use crate::ledger::*;

use super::{
    track::VirtualSubstateId, SubstateOperation, SubstateOperationsReceipt, SubstateParentId,
    SubstateValue,
};

pub enum StateTrackParent {
    SubstateStore(Rc<dyn ReadableSubstateStore>),
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

    pub fn get_substate(&mut self, address: &Address) -> Option<SubstateValue> {
        // TODO: it's very inconvenient to encode & decode. We should consider making SubstateValue cloneable
        //  or have proper borrow mechanism implemented.

        self.substates
            .entry(address.clone())
            .or_insert_with(|| match &mut self.parent {
                StateTrackParent::SubstateStore(store, ..) => {
                    store.get_substate(address).map(|s| s.value)
                }
                StateTrackParent::StateTrack(track) => {
                    track.get_substate(address).map(|s| scrypto_encode(&s))
                }
            })
            .as_ref()
            .map(|s| scrypto_decode(&s).unwrap())
    }

    pub fn put_substate(&mut self, address: Address, substate: SubstateValue) {
        self.substates
            .insert(address, Some(scrypto_encode(&substate)));
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
            store_instructions.push(SubstateOperation::VirtualUp(space.clone()));
        }

        for (address, substate) in self.substates.drain() {
            if let Some(substate) = substate {
                match &address {
                    Address::NonFungible(resource_address, key) => {
                        let parent_address = Address::NonFungibleSet(*resource_address);

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

                        store_instructions.push(SubstateOperation::Up(address, substate));
                    }
                    Address::KeyValueStoreEntry(kv_store_id, key) => {
                        let parent_address = Address::KeyValueStore(*kv_store_id);

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

                        store_instructions.push(SubstateOperation::Up(address, substate));
                    }
                    _ => {
                        if let Some(previous_substate_id) =
                            Self::get_substate_id(&self.parent, &address)
                        {
                            store_instructions.push(SubstateOperation::Down(previous_substate_id));
                        }
                        store_instructions.push(SubstateOperation::Up(address, substate));
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
