use radix_engine_derive::ScryptoSbor;
use utils::rust::boxed::Box;
use utils::rust::collections::IndexMap;
use utils::rust::vec::Vec;

pub type DatabaseUpdates = IndexMap<Vec<u8>, IndexMap<Vec<u8>, DatabaseUpdate>>;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum DatabaseUpdate {
    Set(Vec<u8>),
    Delete,
}

/// Represents the interface between Track and a database vendor.
pub trait SubstateDatabase {
    /// Reads a substate of the given node module.
    ///
    /// [`Option::None`] is returned if missing.
    fn get_substate(&self, index_id: &Vec<u8>, key: &Vec<u8>) -> Option<Vec<u8>>;

    /// Returns a lexicographical sorted iterator over the substates of an index
    fn list_substates(
        &self,
        index_id: &Vec<u8>,
    ) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + '_>;
}

/// Interface for committing changes into a substate database.
pub trait CommittableSubstateDatabase {
    /// Commits state changes to the database.
    ///
    /// An error is thrown in case of invalid module ID.
    fn commit(&mut self, state_changes: &DatabaseUpdates);
}

/// Interface for listing nodes within a substate database.
pub trait ListableSubstateDatabase {
    fn list_nodes(&self) -> Vec<Vec<u8>>;
}
