use crate::{model::NonFungibleSubstate, types::*};

#[derive(Debug)]
pub struct NonFungibleStore {
    pub loaded_non_fungibles: HashMap<NonFungibleLocalId, NonFungibleSubstate>,
}

impl NonFungibleStore {
    pub fn new() -> Self {
        Self {
            loaded_non_fungibles: HashMap::new(),
        }
    }

    pub fn get(&mut self, id: &NonFungibleLocalId) -> Option<&NonFungibleSubstate> {
        self.loaded_non_fungibles.get(id)
    }

    pub fn put(&mut self, id: NonFungibleLocalId, non_fungible: NonFungibleSubstate) {
        self.loaded_non_fungibles.insert(id, non_fungible);
    }
}
