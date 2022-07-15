use sbor::rust::collections::*;

use crate::engine::Address;
use crate::ledger::*;

pub struct SubstateStoreTrack {
    substate_store: Box<dyn ReadableSubstateStore>,
    substates: HashMap<Address, Substate>,
    spaces: HashMap<Address, PhysicalSubstateId>,
}

impl SubstateStoreTrack {
    // TODO: produce substate update receipt
}

impl ReadableSubstateStore for SubstateStoreTrack {
    fn get_substate(&self, address: &Address) -> Option<Substate> {
        self.substates
            .get(address)
            .cloned()
            .or_else(|| self.substate_store.get_substate(address))
    }
    fn get_space(&mut self, address: &Address) -> Option<PhysicalSubstateId> {
        self.spaces
            .get(address)
            .cloned()
            .or_else(|| self.substate_store.get_space(address))
    }
}

impl WriteableSubstateStore for SubstateStoreTrack {
    fn put_substate(&mut self, address: Address, substate: Substate) {
        self.substates.insert(address, substate);
    }

    fn put_space(&mut self, address: Address, phys_id: PhysicalSubstateId) {
        self.spaces.insert(address, phys_id);
    }
}
