use radix_engine_interface::api::LockFlags;
use crate::types::*;
use radix_engine_interface::types::*;

/// Error when acquiring a lock.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AcquireLockError {
    NotFound(NodeId, ModuleNumber, SubstateKey),
    SubstateLocked(NodeId, ModuleNumber, SubstateKey),
    LockUnmodifiedBaseOnNewSubstate(NodeId, ModuleNumber, SubstateKey),
    LockUnmodifiedBaseOnOnUpdatedSubstate(NodeId, ModuleNumber, SubstateKey),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SetSubstateError {
    SubstateLocked(NodeId, ModuleNumber, SubstateKey),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TakeSubstateError {
    SubstateLocked(NodeId, ModuleNumber, SubstateKey),
}

pub type NodeSubstates = BTreeMap<ModuleNumber, BTreeMap<SubstateKey, IndexedScryptoValue>>;

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
        module_id: ModuleNumber,
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
        module_id: ModuleNumber,
        substate_key: &SubstateKey,
    ) -> Result<Option<IndexedScryptoValue>, TakeSubstateError>;

    fn scan_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleNumber,
        count: u32,
    ) -> Vec<IndexedScryptoValue>;

    fn take_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleNumber,
        count: u32,
    ) -> Vec<IndexedScryptoValue>;

    fn scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleNumber,
        count: u32,
    ) -> Vec<IndexedScryptoValue>;

    /// Acquires a lock over a substate.
    /// Returns tuple of lock handle id and information if particular substate
    /// is locked for the first time during transaction execution.
    fn acquire_lock(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<(u32, bool), AcquireLockError> {
        self.acquire_lock_virtualize(node_id, module_id, substate_key, flags, || None)
    }

    fn acquire_lock_virtualize<F: FnOnce() -> Option<IndexedScryptoValue>>(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        virtualize: F,
    ) -> Result<(u32, bool), AcquireLockError>;

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
