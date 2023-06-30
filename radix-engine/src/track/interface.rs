use crate::types::*;
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::types::*;

/// Error when acquiring a lock.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AcquireLockError {
    NotFound(NodeId, PartitionNumber, SubstateKey),
    SubstateLocked(NodeId, PartitionNumber, SubstateKey),
    LockUnmodifiedBaseOnNewSubstate(NodeId, PartitionNumber, SubstateKey),
    LockUnmodifiedBaseOnOnUpdatedSubstate(NodeId, PartitionNumber, SubstateKey),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SetSubstateError {
    SubstateLocked(NodeId, PartitionNumber, SubstateKey),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TakeSubstateError {
    SubstateLocked(NodeId, PartitionNumber, SubstateKey),
}

pub type NodeSubstates = BTreeMap<PartitionNumber, BTreeMap<SubstateKey, IndexedScryptoValue>>;

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
    /// - If the partition is invalid
    fn create_node(&mut self, node_id: NodeId, node_substates: NodeSubstates) -> StoreAccessInfo;

    /// Inserts a substate into the substate store.
    ///
    /// Clients must ensure the `node_id`/`partition_num` is a node which has been created; otherwise, the behavior
    /// is undefined.
    fn set_substate(
        &mut self,
        node_id: NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    ) -> Result<StoreAccessInfo, SetSubstateError>;

    /// Deletes a substate from the substate store.
    ///
    /// Clients must ensure the `node_id`/`partition_num` is a node which has been created;
    /// Clients must ensure this isn't called on a virtualized partition;
    /// Otherwise, the behavior is undefined.
    ///
    /// Returns tuple of substate and boolean which is true for the first database access.
    fn take_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(Option<IndexedScryptoValue>, StoreAccessInfo), TakeSubstateError>;

    /// Returns tuple of substate vector and boolean which is true for the first database access.
    fn scan_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> (Vec<IndexedScryptoValue>, StoreAccessInfo);

    /// Returns tuple of substate vector and boolean which is true for the first database access.
    fn take_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> (Vec<IndexedScryptoValue>, StoreAccessInfo);

    /// Returns tuple of substate vector and boolean which is true for the first database access.
    fn scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> (Vec<IndexedScryptoValue>, StoreAccessInfo);

    /// Acquires a lock over a substate.
    /// Returns tuple of lock handle id and information if particular substate
    /// is locked for the first time during transaction execution.
    fn acquire_lock(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<(u32, StoreAccessInfo), AcquireLockError> {
        self.acquire_lock_virtualize(node_id, partition_num, substate_key, flags, || None)
    }

    fn acquire_lock_virtualize<F: FnOnce() -> Option<IndexedScryptoValue>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        virtualize: F,
    ) -> Result<(u32, StoreAccessInfo), AcquireLockError>;

    /// Releases a lock.
    ///
    /// # Panics
    /// - If the lock handle is invalid.
    fn close_substate(&mut self, handle: u32) -> StoreAccessInfo;

    /// Reads a substate of the given node partition.
    ///
    /// # Panics
    /// - If the lock handle is invalid
    fn read_substate(&mut self, handle: u32) -> (&IndexedScryptoValue, StoreAccessInfo);

    /// Updates a substate.
    ///
    /// # Panics
    /// - If the lock handle is invalid;
    /// - If the lock handle is not associated with WRITE permission
    fn update_substate(
        &mut self,
        handle: u32,
        substate_value: IndexedScryptoValue,
    ) -> StoreAccessInfo;

    /// Note: unstable interface, for intent transaction tracker only
    fn delete_partition(&mut self, node_id: &NodeId, partition_num: PartitionNumber);

    /// Return the commit info
    fn get_commit_info(&mut self) -> StoreCommitInfo;
}

pub type StoreAccessInfo = Vec<StoreAccess>;

#[derive(Debug, Clone, Copy)]
pub enum StoreAccess {
    /// Some substate was read from database.
    ReadFromDb(usize),
    /// Non-existent substate was read from database.
    ReadFromDbNotFound,
    /// A new entry has been added to track
    /// System limits how many items that can be tracked.
    NewEntryInTrack,
}

pub type StoreCommitInfo = Vec<StoreCommit>;

#[derive(Debug, Clone)]
pub enum StoreCommit {
    Insert {
        node_id: NodeId,
        size: usize,
    },
    Update {
        node_id: NodeId,
        size: usize,
        old_size: usize,
    },
    Delete {
        node_id: NodeId,
        old_size: usize,
    },
}

impl StoreCommit {
    pub fn node_id(&self) -> &NodeId {
        match self {
            StoreCommit::Insert { node_id, .. }
            | StoreCommit::Update { node_id, .. }
            | StoreCommit::Delete { node_id, .. } => node_id,
        }
    }
}
