use sbor::rust::collections::*;
use transaction::validation::IdAllocator;

use crate::engine::Address;
use crate::ledger::*;
use scrypto::crypto::Hash;

pub enum StateTrackParent {
    SubstateStore(Box<dyn ReadableSubstateStore>),
    StateTrack(Box<StateTrack>),
}

pub struct StateTrack {
    /// The parent state track
    parent: StateTrackParent,
    /// For substate id generation
    transaction_hash: Hash,
    /// For substate id generation
    id_allocator: IdAllocator,
    /// Loaded or created substates
    substates: HashMap<Address, Option<Substate>>,
    /// Loaded or created spaces
    spaces: HashMap<Address, Option<()>>,
}

impl StateTrack {
    // TODO: produce substate update receipt

    pub fn new(
        parent: StateTrackParent,
        transaction_hash: Hash,
        id_allocator: IdAllocator,
    ) -> Self {
        Self {
            parent,
            transaction_hash,
            id_allocator,
            substates: HashMap::new(),
            spaces: HashMap::new(),
        }
    }

    fn get_substate(&mut self, address: &Address) -> Option<Substate> {
        self.substates
            .entry(address.clone())
            .or_insert_with(|| match self.parent {
                StateTrackParent::SubstateStore(store) => store.get_substate(address),
                StateTrackParent::StateTrack(track) => track.get_substate(address),
            })
            .clone()
    }

    fn get_space(&mut self, address: &Address) -> Option<()> {
        self.spaces
            .entry(address.clone())
            .or_insert_with(|| match self.parent {
                StateTrackParent::SubstateStore(store) => store.get_space(address).map(|_| ()),
                StateTrackParent::StateTrack(track) => track.get_space(address),
            })
            .clone()
    }

    fn put_substate(&mut self, address: Address, substate: Substate) {
        self.substates.insert(address, Some(substate));
    }

    fn put_space(&mut self, address: Address) {
        self.spaces.insert(address, Some(()));
    }
}
