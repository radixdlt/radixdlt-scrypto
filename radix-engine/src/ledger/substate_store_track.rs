use sbor::rust::collections::*;

use crate::engine::Address;
use crate::ledger::*;

pub struct SubstateStoreTrack {
    parent: Box<dyn ReadableSubstateStore>,
    substates: HashMap<Address, Option<Vec<u8>>>,
    spaces: HashMap<Address, bool>,
}

impl SubstateStoreTrack {
    // TODO: produce substate update receipt
    
    pub fn new(parent: Box<dyn ReadableSubstateStore>) -> Self {
        Self {
            parent,
            substates: HashMap::new(),
            spaces: HashMap::new(),
        }
    }
}

impl ReadableSubstateStore for SubstateStoreTrack {
    fn get_substate(&mut self, address: &Address) -> Option<Vec<u8>> {
        self.substates
            .entry(address.clone())
            .or_insert_with(|| self.parent.get_substate(address))
            .clone()
    }
    fn get_space(&mut self, address: &Address) -> bool {
        self.spaces
            .entry(address.clone())
            .or_insert_with(|| self.parent.get_space(address))
            .clone()
    }
}

impl WriteableSubstateStore for SubstateStoreTrack {
    fn put_substate(&mut self, address: Address, substate: Vec<u8>) {
        self.substates.insert(address, Some(substate));
    }

    fn put_space(&mut self, address: Address) {
        self.spaces.insert(address, true);
    }
}
