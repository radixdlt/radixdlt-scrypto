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

use crate::types::*;
use radix_engine_interface::api::LockFlags;

/// The unique identifier of a (stored) node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId([u8; Self::LENGTH]);

impl NodeId {
    pub const LENGTH: usize = 27;

    pub fn new(entity_byte: u8, random_bytes: &[u8; Self::LENGTH - 1]) -> Self {
        let mut buf = [0u8; Self::LENGTH];
        buf[0] = entity_byte;
        buf[1..random_bytes.len() + 1].copy_from_slice(random_bytes);
        Self(buf)
    }
}

impl AsRef<[u8]> for NodeId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Into<[u8; NodeId::LENGTH]> for NodeId {
    fn into(self) -> [u8; NodeId::LENGTH] {
        self.0
    }
}

/// The unique identifier of a node module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleId(pub u8);

/// The unique identifier of a substate within a node module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SubstateKey(Vec<u8>);

impl SubstateKey {
    pub const MIN_LENGTH: usize = 1;
    pub const MAX_LENGTH: usize = 128;
    pub const MIN: Self = Self(vec![u8::MIN; Self::MIN_LENGTH]);
    pub const MAX: Self = Self(vec![u8::MAX; Self::MAX_LENGTH]);

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        Self::from_vec(slice.to_vec())
    }

    pub fn from_vec(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() < Self::MIN_LENGTH || bytes.len() > Self::MAX_LENGTH {
            None
        } else {
            Some(Self(bytes))
        }
    }
}

impl AsRef<[u8]> for SubstateKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Into<Vec<u8>> for SubstateKey {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

/// Utility function for encoding a substate ID `(NodeId, ModuleId, SubstateKey)` into a `Vec<u8>`,
pub fn encode_substate_id(
    node_id: &NodeId,
    module_id: ModuleId,
    substate_key: &SubstateKey,
) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend(&node_id.0);
    buffer.push(module_id.0);
    match substate_key {
        SubstateKey::Config => {
            buffer.push(0);
        }
        SubstateKey::State(state_id) => {
            buffer.push(1);
            buffer.extend(state_id.as_ref()); // Length is marked by EOF
        }
    }
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
        if let Some(id) = SubstateKey::from_slice(&slice[NodeId::LENGTH + 1..]) {
            return Some((node_id, module_id, SubstateKey::State(id)));
        }
    }
    return None;
}

/// Error when acquiring a lock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcquireLockError {
    NotFound(NodeId, ModuleId, SubstateKey),
    SubstateLocked(NodeId, ModuleId, SubstateKey),
    LockUnmodifiedBaseOnNewSubstate(NodeId, ModuleId, SubstateKey),
    LockUnmodifiedBaseOnOnUpdatedSubstate(NodeId, ModuleId, SubstateKey),
}

/// Represents the interface between Radix Engine and Track.
///
/// In practice, we will likely end up with only one implementation.
///
/// The trait here is for formalizing the interface and intended user flow.
pub trait SubstateStore {
    /// Acquires a lock over a substate.
    ///
    /// # Panics
    /// - If the module ID is invalid
    fn acquire_lock(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
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
    fn get_substate(&self, handle: u32) -> &IndexedScryptoValue;

    /// Updates a substate.
    ///
    /// # Panics
    /// - If the lock handle is invalid, or not associated with WRITE permission
    fn put_substate(&mut self, handle: u32, substate_value: IndexedScryptoValue);

    /// Inserts a substate into the substate store.
    ///
    /// Clients must ensure the `node_id` is new and unique; otherwise, the behavior is undefined.
    ///
    /// # Panics
    /// - If the module ID is invalid
    fn insert_substate(
        &mut self,
        node_id: NodeId,
        module_id: ModuleId,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    );

    /// Returns an iterator over substates within the given substate module.
    ///
    /// In case the module does not exist, an empty iterator is returned.
    ///
    /// # Panics
    /// - If the module ID is invalid
    /// - If iteration is not enabled for the module
    /// - If any of the substates within the module is WRITE locked
    fn list_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> dyn Iterator<Item = (SubstateKey, IndexedScryptoValue)>;

    /// Reverts all non force write changes.
    ///
    /// Note that dependencies will never be reverted.
    fn revert_non_force_write_changes(&mut self);

    /// Finalizes changes captured by this substate store.
    ///
    ///  Returns the state changes and dependencies.
    fn finalize(self) -> (StateChanges, StateDependencies);
}

pub struct StateChanges {
    pub substate_changes: BTreeMap<(NodeId, ModuleId, SubstateKey), StateChange>,
}

pub enum StateChange {
    /// Creates or updates a substate.
    Upsert(IndexedScryptoValue),
    /*
    /// Deletes a substate.
    Delete,
    /// Edits an element of a substate, identified by an SBOR path.
    Edit,
    */
}

pub struct StateDependencies {
    /// The substates that were read.
    pub substate_reads: BTreeMap<(NodeId, ModuleId, SubstateKey), Option<u32>>,
    /// The modules which have been iterated.
    pub module_reads: BTreeMap<(NodeId, ModuleId), Option<Hash>>,
}

/// The configuration of a node module.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
pub struct ModuleConfig {
    /// When activated, the store will allow LIST over the substates within the module.
    iteration_enabled: bool,
}

/// Error when initializing a database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitError {
    /// The database is already initialized with a different configuration.
    AlreadyInitializedWithDifferentConfig,
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
    /// Initializes the database with the given config.
    ///
    /// If the database is already initialized, implementation of this method will check if
    /// the set configuration matches the expected configuration and return an error if they do
    /// not match.
    fn init(&self, config: BTreeMap<ModuleId, ModuleConfig>) -> Result<(), InitError>;

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
    ) -> Result<Option<(Vec<u8>, u32)>, GetSubstateError>;

    /// Returns an iterator over substates within the given substate module, and the module's root hash.
    ///
    /// In case the module does not exist, an empty iterator is returned.
    ///
    /// If iteration is not enabled for the module ID or the module ID is invalid, an error is thrown.
    fn list_substates(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Result<(dyn Iterator<Item = (SubstateKey, Vec<u8>)>, Hash), ListSubstatesError>;
}

/// Interface for committing changes into a substate database.
pub trait CommittableSubstateDatabase {
    /// Commits state changes to the database.
    ///
    /// An error is thrown in case of invalid module ID.
    fn commit(&mut self, state_changes: StateChanges) -> Result<(), CommitError>;
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
        let substate_key = SubstateKey::State(SubstateKey::from_vec(vec![3]).unwrap());
        let substate_id = encode_substate_id(&node_id, module_id, &substate_key);
        assert_eq!(
            substate_id,
            vec![
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, // node id
                2, // module id
                1, 3, // substate key
            ]
        );
        assert_eq!(
            decode_substate_id(&substate_id),
            Some((node_id, module_id, substate_key))
        )
    }
}
