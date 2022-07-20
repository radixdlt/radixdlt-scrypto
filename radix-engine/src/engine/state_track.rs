use core::ops::RangeFull;

use indexmap::IndexSet;
use sbor::rust::collections::*;
use sbor::rust::rc::Rc;
use sbor::rust::vec::Vec;
use scrypto::buffer::{scrypto_decode, scrypto_encode};

use crate::engine::Address;
use crate::ledger::*;

use super::{
    track::VirtualSubstateId, SubstateOperation, SubstateOperationsReceipt, SubstateParentId,
    SubstateValue,
};

pub trait StateTrack {
    fn get_substate(&mut self, address: &Address) -> Option<SubstateValue>;

    fn put_substate(&mut self, address: Address, substate: SubstateValue);

    fn put_space(&mut self, address: Address);
}

/// Keeps track of state changes that that are non-reversible, such as fee payments
pub struct BaseStateTrack {
    /// The parent state track
    substate_store: Rc<dyn ReadableSubstateStore>,
    /// Substates either created during the transaction or loaded from substate store
    substates: HashMap<Address, Option<Vec<u8>>>,
    /// Spaces created during the transaction
    spaces: IndexSet<Address>,
}

/// Keeps track of state changes that may be rolled back according to transaction status
pub struct AppStateTrack {
    /// The parent state track
    base_state_track: BaseStateTrack,
    /// Substates either created during the transaction or loaded from the base state track
    substates: HashMap<Address, Option<Vec<u8>>>,
    /// Spaces created during the transaction
    spaces: IndexSet<Address>,
}

impl BaseStateTrack {
    pub fn new(substate_store: Rc<dyn ReadableSubstateStore>) -> Self {
        Self {
            substate_store,
            substates: HashMap::new(),
            spaces: IndexSet::new(),
        }
    }

    fn get_substate_id(
        substate_store: &Rc<dyn ReadableSubstateStore>,
        address: &Address,
    ) -> Option<PhysicalSubstateId> {
        substate_store.get_substate(&address).map(|s| s.phys_id)
    }

    fn get_space_substate_id(
        substate_store: &Rc<dyn ReadableSubstateStore>,
        address: &Address,
    ) -> Option<PhysicalSubstateId> {
        substate_store.get_space(&address)
    }

    fn get_substate_parent_id(
        spaces: &IndexSet<Address>,
        substate_store: &Rc<dyn ReadableSubstateStore>,
        space_address: &Address,
    ) -> SubstateParentId {
        if let Some(index) = spaces.get_index_of(space_address) {
            SubstateParentId::New(index)
        } else {
            let substate_id = Self::get_space_substate_id(substate_store, space_address)
                .expect("Attempted to locate non-existing space");
            SubstateParentId::Exists(substate_id)
        }
    }

    pub fn to_receipt(mut self) -> SubstateOperationsReceipt {
        let mut store_instructions = Vec::new();

        // Must be put in front of substate changes to maintain valid parent ids.
        for space in &self.spaces {
            store_instructions.push(SubstateOperation::VirtualUp(space.clone()));
        }

        for (address, substate) in self.substates.drain() {
            if let Some(substate) = substate {
                match &address {
                    Address::NonFungible(resource_address, key) => {
                        if let Some(existing_substate_id) =
                            Self::get_substate_id(&self.substate_store, &address)
                        {
                            store_instructions.push(SubstateOperation::Down(existing_substate_id));
                        } else {
                            let parent_address = Address::NonFungibleSpace(*resource_address);
                            let parent_id = Self::get_substate_parent_id(
                                &self.spaces,
                                &self.substate_store,
                                &parent_address,
                            );
                            let virtual_substate_id = VirtualSubstateId(parent_id, key.clone());
                            store_instructions
                                .push(SubstateOperation::VirtualDown(virtual_substate_id));
                        }

                        store_instructions.push(SubstateOperation::Up(address, substate));
                    }
                    Address::KeyValueStoreEntry(kv_store_id, key) => {
                        if let Some(existing_substate_id) =
                            Self::get_substate_id(&self.substate_store, &address)
                        {
                            store_instructions.push(SubstateOperation::Down(existing_substate_id));
                        } else {
                            let parent_address = Address::KeyValueStoreSpace(*kv_store_id);
                            let parent_id = Self::get_substate_parent_id(
                                &self.spaces,
                                &self.substate_store,
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
                            Self::get_substate_id(&self.substate_store, &address)
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

#[derive(Debug)]
pub enum WriteThroughOperationError {
    ValueAlreadyTouched,
}

impl AppStateTrack {
    pub fn new(base_state_track: BaseStateTrack) -> Self {
        Self {
            base_state_track,
            substates: HashMap::new(),
            spaces: IndexSet::new(),
        }
    }

    pub fn get_substate_from_base(
        &mut self,
        address: &Address,
    ) -> Result<SubstateValue, WriteThroughOperationError> {
        if self.substates.contains_key(address) {
            return Err(WriteThroughOperationError::ValueAlreadyTouched);
        }

        Ok(self
            .base_state_track
            .get_substate(address)
            .expect("Attempted to load non-existing substate with write-through mode"))
    }

    pub fn put_substate_to_base(&mut self, address: Address, substate: SubstateValue) {
        assert!(!self.substates.contains_key(&address));

        self.base_state_track.put_substate(address, substate);
    }

    /// Flush all changes to base state track
    pub fn flush(&mut self) {
        self.base_state_track
            .substates
            .extend(self.substates.drain());
        self.base_state_track
            .spaces
            .extend(self.spaces.drain(RangeFull));
    }

    /// Unwraps into the base state track
    pub fn unwrap(self) -> BaseStateTrack {
        self.base_state_track
    }
}

impl StateTrack for BaseStateTrack {
    fn get_substate(&mut self, address: &Address) -> Option<SubstateValue> {
        // TODO: consider borrow mechanism to avoid redundant encoding and decoding

        self.substates
            .entry(address.clone())
            .or_insert_with(|| self.substate_store.get_substate(address).map(|s| s.value))
            .as_ref()
            .map(|s| scrypto_decode(&s).unwrap())
    }

    fn put_substate(&mut self, address: Address, substate: SubstateValue) {
        self.substates
            .insert(address, Some(scrypto_encode(&substate)));
    }

    fn put_space(&mut self, address: Address) {
        self.spaces.insert(address);
    }
}

impl StateTrack for AppStateTrack {
    fn get_substate(&mut self, address: &Address) -> Option<SubstateValue> {
        // TODO: consider borrow mechanism to avoid redundant encoding and decoding

        self.substates
            .entry(address.clone())
            .or_insert_with(|| {
                self.base_state_track
                    .get_substate(address)
                    .map(|s| scrypto_encode(&s))
            })
            .as_ref()
            .map(|s| scrypto_decode(&s).unwrap())
    }

    fn put_substate(&mut self, address: Address, substate: SubstateValue) {
        self.substates
            .insert(address, Some(scrypto_encode(&substate)));
    }

    fn put_space(&mut self, address: Address) {
        self.spaces.insert(address);
    }
}
