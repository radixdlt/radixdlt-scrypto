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
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::prelude::*;

// TODO: Add streaming support for `list_substates`

/// Utility function for encoding a substate ID `(NodeId, ModuleId, SubstateKey)` into a `Vec<u8>`,
pub fn encode_substate_id(
    node_id: &NodeId,
    module_id: ModuleId,
    substate_key: &SubstateKey,
) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend(node_id.as_ref());
    buffer.push(module_id.0);
    buffer.extend(substate_key.as_ref()); // Length is marked by EOF
    buffer
}

/// Utility function for decoding a substate ID `(NodeId, ModuleId, SubstateKey)` from a `Vec<u8>`,
pub fn decode_substate_id(slice: &[u8]) -> Option<(NodeId, ModuleId, SubstateKey)> {
    if slice.len() >= NodeId::LENGTH + 1 {
        // Decode node id
        let mut node_id = [0u8; NodeId::LENGTH];
        node_id.copy_from_slice(&slice[0..NodeId::LENGTH]);
        let node_id = NodeId(node_id);

        // Decode module id
        let module_id = ModuleId(slice[NodeId::LENGTH]);

        // Decode substate key
        if let Some(substate_key) = SubstateKey::from_slice(&slice[NodeId::LENGTH + 1..]) {
            return Some((node_id, module_id, substate_key));
        }
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
pub enum DeleteSubstateError {
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

    fn get_substate(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue>;

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
    /// Clients must ensure the `node_id`/`module_id` is a node which has been created; otherwise, the behavior
    /// is undefined.
    fn delete_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Result<Option<IndexedScryptoValue>, DeleteSubstateError>;

    fn scan_sorted(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        count: u32,
    ) -> Vec<(SubstateKey, IndexedScryptoValue)>;

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
pub struct StateUpdates {
    pub substate_changes: IndexMap<(NodeId, ModuleId, SubstateKey), StateUpdate>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum StateUpdate {
    Set(Vec<u8>),
    Delete,
}

/// The configuration of a node module.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
pub struct ModuleConfig {
    /// When activated, the store will allow LIST over the substates within the module.
    pub iteration_enabled: bool,
}

/// Error when listing substates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListSubstatesError {
    /// The module ID is unknown.
    UnknownModuleId,
    /// Iteration is not enabled for the module.
    IterationNotAllowed,
}

/// Error when reading substates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetSubstateError {
    /// The module ID is unknown.
    UnknownModuleId,
}

/// Error when reading substates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitError {
    /// The module ID is unknown.
    UnknownModuleId,
}

/// Represents the interface between Track and a database vendor.
pub trait SubstateDatabase {
    /// Reads a substate of the given node module.
    ///
    /// [`Option::None`] is returned if missing.
    ///
    /// An error is thrown in case of invalid module ID.
    fn get_substate(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Result<Option<Vec<u8>>, GetSubstateError>;

    /// Returns an iterator over substates within the given substate module, and the module's root hash.
    ///
    /// In case the module does not exist, an empty iterator is returned.
    ///
    /// If iteration is not enabled for the module ID or the module ID is invalid, an error is thrown.
    fn list_substates(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        count: u32,
    ) -> Result<Vec<(SubstateKey, Vec<u8>)>, ListSubstatesError>;
}

/// Interface for committing changes into a substate database.
pub trait CommittableSubstateDatabase {
    /// Commits state changes to the database.
    ///
    /// An error is thrown in case of invalid module ID.
    fn commit(&mut self, state_changes: &StateUpdates) -> Result<(), CommitError>;
}

/// Interface for listing nodes within a substate database.
pub trait ListableSubstateDatabase {
    fn list_nodes(&self) -> Vec<NodeId>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_substate_id() {
        let node_id = NodeId([1u8; NodeId::LENGTH]);
        let module_id = ModuleId(2);
        let substate_key = SubstateKey::from_vec(vec![3]).unwrap();
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
}
