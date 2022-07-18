use core::ops::RangeFull;

use sbor::rust::collections::*;
use scrypto::crypto::Hash;
use transaction::validation::IdAllocator;

use crate::engine::Address;
use crate::ledger::*;

use super::{SubstateOperation, SubstateOperationsReceipt};

pub enum StateTrackParent {
    SubstateStore(Box<dyn ReadableSubstateStore>, Hash, IdAllocator),
    StateTrack(Box<StateTrack>),
}

pub struct StateTrack {
    /// The parent state track
    parent: StateTrackParent,
    /// Loaded or created substates
    substates: HashMap<Address, Option<Vec<u8>>>,
    /// Created spaces
    spaces: HashSet<Address>,
}

impl StateTrack {
    // TODO: produce substate update receipt

    pub fn new(parent: StateTrackParent) -> Self {
        Self {
            parent,
            substates: HashMap::new(),
            spaces: HashSet::new(),
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

    pub fn summarize_state_changes(self) -> SubstateOperationsReceipt {
        let mut store_instructions = Vec::new();

        // for substate_id in self.downed_substates {
        //     store_instructions.push(SubstateOperation::Down(substate_id));
        // }
        // for virtual_substate_id in self.down_virtual_substates {
        //     store_instructions.push(SubstateOperation::VirtualDown(virtual_substate_id));
        // }
        // for (address, value) in self.up_substates.drain(RangeFull) {
        //     store_instructions.push(SubstateOperation::Up(address, value.encode()));
        // }
        // for space_address in self.up_virtual_substate_space.drain(RangeFull) {
        //     store_instructions.push(SubstateOperation::VirtualUp(space_address));
        // }

        SubstateOperationsReceipt {
            substate_operations: store_instructions,
        }
    }
}
