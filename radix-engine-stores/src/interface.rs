/*

High-level Abstraction

+-------------------------+
|       Radix Engine      |
|----> SubstateStore <----|
|          Track          |
|---> SubstateDatabase <--|
|         Database        |
+-------------------------+

*/

use radix_engine_interface::api::LockFlags;
use radix_engine_interface::data::scrypto::{scrypto_decode, ScryptoDecode};
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::prelude::*;

pub fn encode_substate_id(index_id: &Vec<u8>, db_key: &Vec<u8>) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend(index_id);
    buffer.extend(db_key); // Length is marked by EOF
    buffer
}

pub fn decode_substate_id(slice: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    if slice.len() >= 26 {
        let index_id = slice[0..26].to_vec();
        let key = slice[26 + 1..].to_vec();

        return Some((index_id, key));
    }

    return None;
}

/// Error when acquiring a lock.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AcquireLockError {
    NotFound(NodeId, ModuleId, SubstateKey),
    SubstateLocked(NodeId, ModuleId, SubstateKey),
    LockUnmodifiedBaseOnNewSubstate(NodeId, ModuleId, SubstateKey),
    LockUnmodifiedBaseOnOnUpdatedSubstate(NodeId, ModuleId, SubstateKey),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SetSubstateError {
    SubstateLocked(NodeId, ModuleId, SubstateKey),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TakeSubstateError {
    SubstateLocked(NodeId, ModuleId, SubstateKey),
}

pub type NodeSubstates = BTreeMap<ModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>;

/// Represents the interface between Radix Engine and Track.
///
/// In practice, we will likely end up with only one implementation.
///
/// The trait here is for formalizing the interface and intended user flow.
pub trait SubstateStore {
    /// Inserts a node into the substate store.
    ///
    /// Clients must ensure the `node_id` is new and unique; otherwise, the behavior is undefined.
    ///
    /// # Panics
    /// - If the module ID is invalid
    fn create_node(&mut self, node_id: NodeId, node_substates: NodeSubstates);

    /// Inserts a substate into the substate store.
    ///
    /// Clients must ensure the `node_id`/`module_id` is a node which has been created; otherwise, the behavior
    /// is undefined.
    fn set_substate(
        &mut self,
        node_id: NodeId,
        module_id: ModuleId,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    ) -> Result<(), SetSubstateError>;

    /// Deletes a substate from the substate store.
    ///
    /// Clients must ensure the `node_id`/`module_id` is a node which has been created;
    /// Clients must ensure this isn't called on a virtualized module;
    /// Otherwise, the behavior is undefined.
    fn take_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Result<Option<IndexedScryptoValue>, TakeSubstateError>;

    fn scan_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        count: u32,
    ) -> Vec<IndexedScryptoValue>;

    fn take_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        count: u32,
    ) -> Vec<IndexedScryptoValue>;

    fn scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        count: u32,
    ) -> Vec<IndexedScryptoValue>;

    /// Acquires a lock over a substate.
    fn acquire_lock(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<u32, AcquireLockError> {
        self.acquire_lock_virtualize(node_id, module_id, substate_key, flags, || None)
    }

    fn acquire_lock_virtualize<F: FnOnce() -> Option<IndexedScryptoValue>>(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
        virtualize: F,
    ) -> Result<u32, AcquireLockError>;

    /// Releases a lock.
    ///
    /// # Panics
    /// - If the lock handle is invalid.
    fn release_lock(&mut self, handle: u32);

    /// Reads a substate of the given node module.
    ///
    /// # Panics
    /// - If the lock handle is invalid
    fn read_substate(&mut self, handle: u32) -> &IndexedScryptoValue;

    /// Updates a substate.
    ///
    /// # Panics
    /// - If the lock handle is invalid;
    /// - If the lock handle is not associated with WRITE permission
    fn update_substate(&mut self, handle: u32, substate_value: IndexedScryptoValue);
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct DatabaseUpdates {
    pub database_updates: IndexMap<Vec<u8>, IndexMap<Vec<u8>, DatabaseUpdate>>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum DatabaseUpdate {
    Set(Vec<u8>),
    Delete,
}

pub trait DatabaseMapper {
    fn map_to_index_id(node_id: &NodeId, module_id: ModuleId) -> Vec<u8>;
    fn map_to_db_key(key: SubstateKey) -> Vec<u8>;
}

/// Represents the interface between Track and a database vendor.
pub trait SubstateDatabase {
    /// Reads a substate of the given node module.
    ///
    /// [`Option::None`] is returned if missing.
    fn get_substate(&self, index_id: &Vec<u8>, key: &Vec<u8>) -> Option<Vec<u8>>;

    /// Returns an iterator over substates within the given substate module
    fn list_substates(
        &self,
        index_id: &Vec<u8>,
    ) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + '_>;

    /// Convenience method for database readers
    fn read_mapped_substate<M: DatabaseMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: SubstateKey,
    ) -> Option<D> {
        self.get_substate(
            &M::map_to_index_id(node_id, module_id),
            &M::map_to_db_key(substate_key),
        )
        .map(|buf| scrypto_decode(&buf).unwrap())
    }

    /// Convenience method for database readers
    fn list_mapped_substates<M: DatabaseMapper>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + '_> {
        self.list_substates(&M::map_to_index_id(node_id, module_id))
    }
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
    fn list_nodes(&self) -> Vec<NodeId>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    #[test]
    fn test_encode_decode_substate_id() {
        let node_id = NodeId([1u8; NodeId::LENGTH]);
        let module_id = ModuleId(2);
        let substate_key = SubstateKey::Map(vec![3]);
        let substate_id = encode_substate_id(&node_id, module_id, &substate_key);
        assert_eq!(
            substate_id,
            vec![
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, // node id
                2, // module id
                3, // substate key
            ]
        );
        assert_eq!(
            decode_substate_id(&substate_id),
            Some((node_id, module_id, substate_key))
        )
    }
     */
}
