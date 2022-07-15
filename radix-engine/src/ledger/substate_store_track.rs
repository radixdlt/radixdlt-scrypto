use sbor::rust::collections::*;
use sbor::rust::vec::Vec;

use crate::ledger::*;

pub struct SubstateStoreTrack {
    substate_store: Box<dyn ReadableSubstateStore>,
    substates: HashMap<Vec<u8>, Substate>,
    spaces: HashMap<Vec<u8>, PhysicalSubstateId>,
}

impl SubstateStoreTrack {}

impl ReadableSubstateStore for SubstateStoreTrack {
    fn get_substate(&self, address: &[u8]) -> Option<Substate> {
        self.substates
            .get(address)
            .cloned()
            .or_else(|| self.substate_store.get_substate(address))
    }
    fn get_space(&mut self, address: &[u8]) -> Option<PhysicalSubstateId> {
        self.spaces
            .get(address)
            .cloned()
            .or_else(|| self.substate_store.get_space(address))
    }
}

impl WriteableSubstateStore for SubstateStoreTrack {
    fn put_substate(&mut self, address: &[u8], substate: Substate) {
        self.substates.insert(address.to_vec(), substate);
    }

    fn put_space(&mut self, address: &[u8], phys_id: PhysicalSubstateId) {
        self.spaces.insert(address.to_vec(), phys_id);
    }
}
