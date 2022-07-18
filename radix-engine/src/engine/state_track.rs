use sbor::rust::collections::*;
use scrypto::crypto::Hash;
use transaction::validation::IdAllocator;

use crate::engine::Address;
use crate::ledger::*;

pub enum StateTrackParent {
    SubstateStore(Box<dyn ReadableSubstateStore>, Hash, IdAllocator),
    StateTrack(Box<StateTrack>),
}

pub struct StateTrack {
    /// The parent state track
    parent: StateTrackParent,
    /// Loaded or created substates
    substates: HashMap<Address, Option<Vec<u8>>>,
    /// Loaded or created spaces
    spaces: HashMap<Address, Option<()>>,
}

impl StateTrack {
    // TODO: produce substate update receipt

    pub fn new(parent: StateTrackParent) -> Self {
        Self {
            parent,
            substates: HashMap::new(),
            spaces: HashMap::new(),
        }
    }

    pub fn get_substate(&mut self, address: &Address) -> Option<Vec<u8>> {
        self.substates
            .entry(address.clone())
            .or_insert_with(|| match self.parent {
                StateTrackParent::SubstateStore(store, ..) => {
                    store.get_substate(address).map(|s| s.value)
                }
                StateTrackParent::StateTrack(track) => track.get_substate(address),
            })
            .clone()
    }

    pub fn get_space(&mut self, address: &Address) -> Option<()> {
        self.spaces
            .entry(address.clone())
            .or_insert_with(|| match self.parent {
                StateTrackParent::SubstateStore(store, ..) => store.get_space(address).map(|_| ()),
                StateTrackParent::StateTrack(track) => track.get_space(address),
            })
            .clone()
    }

    pub fn put_substate(&mut self, address: Address, substate: Vec<u8>) {
        self.substates.insert(address, Some(substate));
    }

    pub fn put_space(&mut self, address: Address) {
        self.spaces.insert(address, Some(()));
    }
}
