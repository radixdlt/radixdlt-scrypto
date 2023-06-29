use crate::track::interface::{
    AcquireLockError, NodeSubstates, SetSubstateError, StoreAccess, StoreAccessInfo, SubstateStore,
    TakeSubstateError,
};
use crate::track::utils::OverlayingIterator;
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::types::*;
use radix_engine_store_interface::interface::DbPartitionKey;
use radix_engine_store_interface::{
    db_key_mapper::DatabaseKeyMapper,
    interface::{DatabaseUpdate, DatabaseUpdates, DbSortKey, PartitionEntry, SubstateDatabase},
};
use sbor::rust::collections::btree_map::Entry;
use sbor::rust::iter::empty;
use sbor::rust::mem;

#[derive(Default, Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct StateUpdates {
    pub database_updates: DatabaseUpdates,
    pub system_updates: SystemUpdates,
    /// Unstable, for transaction tracker only; Must be applied after committing the updates above.
    /// TODO: if time allows, consider merging it into database/system updates.
    pub partition_deletions: IndexSet<DbPartitionKey>,
}
pub type SystemUpdates = IndexMap<(NodeId, PartitionNumber), IndexMap<SubstateKey, DatabaseUpdate>>;

pub struct SubstateLockError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
pub enum SubstateLockState {
    Read(usize),
    Write,
}

impl SubstateLockState {
    pub fn no_lock() -> Self {
        Self::Read(0)
    }

    pub fn is_locked(&self) -> bool {
        !matches!(self, SubstateLockState::Read(0usize))
    }

    pub fn try_lock(&mut self, flags: LockFlags) -> Result<(), SubstateLockError> {
        match self {
            SubstateLockState::Read(n) => {
                if flags.contains(LockFlags::MUTABLE) {
                    if *n != 0 {
                        return Err(SubstateLockError);
                    }
                    *self = SubstateLockState::Write;
                } else {
                    *n = *n + 1;
                }
            }
            SubstateLockState::Write => {
                return Err(SubstateLockError);
            }
        }

        Ok(())
    }

    fn unlock(&mut self) {
        match self {
            SubstateLockState::Read(n) => {
                *n = *n - 1;
            }
            SubstateLockState::Write => {
                *self = SubstateLockState::no_lock();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeSubstate {
    pub value: IndexedScryptoValue,
    lock_state: SubstateLockState,
}

impl RuntimeSubstate {
    fn new(value: IndexedScryptoValue) -> Self {
        Self {
            value,
            lock_state: SubstateLockState::no_lock(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ReadOnly {
    NonExistent,
    Existent(RuntimeSubstate),
}

#[derive(Clone, Debug)]
pub enum Write {
    Update(RuntimeSubstate),
    Delete,
}

impl Write {
    fn into_value(self) -> Option<IndexedScryptoValue> {
        match self {
            Write::Update(substate) => Some(substate.value),
            Write::Delete => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TrackedSubstateKey {
    pub substate_key: SubstateKey,
    pub tracked: TrackedKey,
}

// TODO: Add new virtualized
#[derive(Clone, Debug)]
pub enum TrackedKey {
    New(RuntimeSubstate),
    ReadOnly(ReadOnly),
    ReadExistAndWrite(IndexedScryptoValue, Write),
    ReadNonExistAndWrite(RuntimeSubstate),
    WriteOnly(Write),
    Garbage,
}

impl TrackedKey {
    pub fn get_runtime_substate_mut(&mut self) -> Option<&mut RuntimeSubstate> {
        match self {
            TrackedKey::New(substate)
            | TrackedKey::WriteOnly(Write::Update(substate))
            | TrackedKey::ReadOnly(ReadOnly::Existent(substate))
            | TrackedKey::ReadExistAndWrite(_, Write::Update(substate))
            | TrackedKey::ReadNonExistAndWrite(substate) => Some(substate),

            TrackedKey::WriteOnly(Write::Delete)
            | TrackedKey::ReadExistAndWrite(_, Write::Delete)
            | TrackedKey::ReadOnly(ReadOnly::NonExistent)
            | TrackedKey::Garbage => None,
        }
    }

    pub fn get(&self) -> Option<&IndexedScryptoValue> {
        match self {
            TrackedKey::New(substate)
            | TrackedKey::WriteOnly(Write::Update(substate))
            | TrackedKey::ReadOnly(ReadOnly::Existent(substate))
            | TrackedKey::ReadExistAndWrite(_, Write::Update(substate))
            | TrackedKey::ReadNonExistAndWrite(substate) => Some(&substate.value),
            TrackedKey::WriteOnly(Write::Delete)
            | TrackedKey::ReadExistAndWrite(_, Write::Delete)
            | TrackedKey::ReadOnly(ReadOnly::NonExistent)
            | TrackedKey::Garbage => None,
        }
    }

    pub fn set(&mut self, value: IndexedScryptoValue) {
        match self {
            TrackedKey::Garbage => {
                *self = TrackedKey::WriteOnly(Write::Update(RuntimeSubstate::new(value)));
            }
            TrackedKey::New(substate)
            | TrackedKey::WriteOnly(Write::Update(substate))
            | TrackedKey::ReadExistAndWrite(_, Write::Update(substate))
            | TrackedKey::ReadNonExistAndWrite(substate) => {
                substate.value = value;
            }
            TrackedKey::ReadOnly(ReadOnly::NonExistent) => {
                let new_tracked = TrackedKey::ReadNonExistAndWrite(RuntimeSubstate::new(value));
                let mut old = mem::replace(self, new_tracked);
                self.get_runtime_substate_mut().unwrap().lock_state =
                    old.get_runtime_substate_mut().unwrap().lock_state;
            }
            TrackedKey::ReadOnly(ReadOnly::Existent(old)) => {
                let new_tracked = TrackedKey::ReadExistAndWrite(
                    old.value.clone(),
                    Write::Update(RuntimeSubstate::new(value)),
                );
                let mut old = mem::replace(self, new_tracked);
                self.get_runtime_substate_mut().unwrap().lock_state =
                    old.get_runtime_substate_mut().unwrap().lock_state;
            }
            TrackedKey::ReadExistAndWrite(_, write @ Write::Delete)
            | TrackedKey::WriteOnly(write @ Write::Delete) => {
                *write = Write::Update(RuntimeSubstate::new(value));
            }
        };
    }

    pub fn take(&mut self) -> Option<IndexedScryptoValue> {
        match self {
            TrackedKey::Garbage => None,
            TrackedKey::New(..) => {
                let old = mem::replace(self, TrackedKey::Garbage);
                old.into_value()
            }
            TrackedKey::WriteOnly(_) => {
                let old = mem::replace(self, TrackedKey::WriteOnly(Write::Delete));
                old.into_value()
            }
            TrackedKey::ReadExistAndWrite(_, write) => {
                let write = mem::replace(write, Write::Delete);
                write.into_value()
            }
            TrackedKey::ReadNonExistAndWrite(..) => {
                let old = mem::replace(self, TrackedKey::ReadOnly(ReadOnly::NonExistent));
                old.into_value()
            }
            TrackedKey::ReadOnly(ReadOnly::Existent(v)) => {
                let new_tracked = TrackedKey::ReadExistAndWrite(v.value.clone(), Write::Delete);
                let old = mem::replace(self, new_tracked);
                old.into_value()
            }
            TrackedKey::ReadOnly(ReadOnly::NonExistent) => None,
        }
    }

    fn revert_writes(&mut self) {
        match self {
            TrackedKey::ReadOnly(..) | TrackedKey::Garbage => {}
            TrackedKey::New(..) | TrackedKey::WriteOnly(_) => {
                *self = TrackedKey::Garbage;
            }
            TrackedKey::ReadExistAndWrite(read, _) => {
                *self =
                    TrackedKey::ReadOnly(ReadOnly::Existent(RuntimeSubstate::new(read.clone())));
            }
            TrackedKey::ReadNonExistAndWrite(..) => {
                *self = TrackedKey::ReadOnly(ReadOnly::NonExistent);
            }
        }
    }

    pub fn into_value(self) -> Option<IndexedScryptoValue> {
        match self {
            TrackedKey::New(substate)
            | TrackedKey::WriteOnly(Write::Update(substate))
            | TrackedKey::ReadOnly(ReadOnly::Existent(substate))
            | TrackedKey::ReadNonExistAndWrite(substate)
            | TrackedKey::ReadExistAndWrite(_, Write::Update(substate)) => Some(substate.value),
            TrackedKey::WriteOnly(Write::Delete)
            | TrackedKey::ReadExistAndWrite(_, Write::Delete)
            | TrackedKey::ReadOnly(ReadOnly::NonExistent)
            | TrackedKey::Garbage => None,
        }
    }
}

#[derive(Debug)]
pub struct TrackedPartition {
    pub substates: BTreeMap<DbSortKey, TrackedSubstateKey>,
    pub range_read: u32,
}

impl TrackedPartition {
    pub fn new() -> Self {
        Self {
            substates: BTreeMap::new(),
            range_read: 0,
        }
    }

    pub fn new_with_substates(substates: BTreeMap<DbSortKey, TrackedSubstateKey>) -> Self {
        Self {
            substates,
            range_read: 0,
        }
    }

    pub fn revert_writes(&mut self) {
        for tracked_key in &mut self.substates.values_mut() {
            tracked_key.tracked.revert_writes();
        }
    }
}

#[derive(Debug)]
pub struct TrackedNode {
    pub tracked_partitions: IndexMap<PartitionNumber, TrackedPartition>,
    // If true, then all SubstateUpdates under this NodeUpdate must be inserts
    // The extra information, though awkward structurally, makes for a much
    // simpler iteration implementation as long as the invariant is maintained
    pub is_new: bool,
}

impl TrackedNode {
    pub fn new(is_new: bool) -> Self {
        Self {
            tracked_partitions: index_map_new(),
            is_new,
        }
    }

    pub fn revert_writes(&mut self) {
        for (_, tracked_partition) in &mut self.tracked_partitions {
            tracked_partition.revert_writes();
        }
    }
}

pub fn to_state_updates<M: DatabaseKeyMapper>(
    index: IndexMap<NodeId, TrackedNode>,
    deleted_partitions: IndexSet<(NodeId, PartitionNumber)>,
) -> StateUpdates {
    let mut database_updates: DatabaseUpdates = index_map_new();
    let mut system_updates: SystemUpdates = index_map_new();
    for (node_id, tracked_node) in index {
        for (partition_num, tracked_partition) in tracked_node.tracked_partitions {
            let mut db_partition_updates = index_map_new();
            let mut partition_updates = index_map_new();

            for (db_sort_key, tracked) in tracked_partition.substates {
                let update = match tracked.tracked {
                    TrackedKey::ReadOnly(..) | TrackedKey::Garbage => None,
                    TrackedKey::ReadNonExistAndWrite(substate) | TrackedKey::New(substate) => {
                        Some(DatabaseUpdate::Set(substate.value.into()))
                    }
                    TrackedKey::ReadExistAndWrite(_, write) | TrackedKey::WriteOnly(write) => {
                        match write {
                            Write::Delete => Some(DatabaseUpdate::Delete),
                            Write::Update(substate) => {
                                Some(DatabaseUpdate::Set(substate.value.into()))
                            }
                        }
                    }
                };
                if let Some(update) = update {
                    db_partition_updates.insert(db_sort_key, update.clone());
                    partition_updates.insert(tracked.substate_key, update);
                }
            }

            let db_partition_key = M::to_db_partition_key(&node_id, partition_num);
            database_updates.insert(db_partition_key, db_partition_updates);
            system_updates.insert((node_id.clone(), partition_num), partition_updates);
        }
    }

    let partition_deletions = deleted_partitions
        .into_iter()
        .map(|(node_id, partition_num)| M::to_db_partition_key(&node_id, partition_num))
        .collect();

    StateUpdates {
        database_updates,
        system_updates,
        partition_deletions,
    }
}

struct TrackedIter<'a> {
    iter: Box<dyn Iterator<Item = PartitionEntry> + 'a>,
    num_iterations: u32,
}

impl<'a> TrackedIter<'a> {
    fn new(iter: Box<dyn Iterator<Item = PartitionEntry> + 'a>) -> Self {
        Self {
            iter,
            num_iterations: 0u32,
        }
    }
}

impl<'a> Iterator for TrackedIter<'a> {
    type Item = PartitionEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.num_iterations = self.num_iterations + 1;
        self.iter.next()
    }
}
/// Transaction-wide states and side effects
pub struct Track<'s, S: SubstateDatabase, M: DatabaseKeyMapper> {
    substate_db: &'s S,
    tracked_nodes: IndexMap<NodeId, TrackedNode>,
    force_write_tracked_nodes: IndexMap<NodeId, TrackedNode>,
    /// TODO: if time allows, consider merging into tracked nodes.
    deleted_partitions: IndexSet<(NodeId, PartitionNumber)>,

    locks: IndexMap<u32, (NodeId, PartitionNumber, SubstateKey, LockFlags)>,
    next_lock_id: u32,
    phantom_data: PhantomData<M>,
}

impl<'s, S: SubstateDatabase, M: DatabaseKeyMapper> Track<'s, S, M> {
    pub fn new(substate_db: &'s S) -> Self {
        Self {
            substate_db,
            force_write_tracked_nodes: index_map_new(),
            tracked_nodes: index_map_new(),
            deleted_partitions: index_set_new(),
            locks: index_map_new(),
            next_lock_id: 0,
            phantom_data: PhantomData::default(),
        }
    }

    fn new_lock_handle(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> u32 {
        let new_lock = self.next_lock_id;
        self.locks.insert(
            new_lock,
            (*node_id, partition_num, substate_key.clone(), flags),
        );
        self.next_lock_id += 1;
        new_lock
    }

    /// Reverts all non force write changes.
    ///
    /// Note that dependencies will never be reverted.
    pub fn revert_non_force_write_changes(&mut self) {
        self.tracked_nodes
            .retain(|_, tracked_node| !tracked_node.is_new);
        for (_, tracked_node) in &mut self.tracked_nodes {
            tracked_node.revert_writes();
        }

        let force_writes = mem::take(&mut self.force_write_tracked_nodes);

        for (node_id, force_track_node) in force_writes {
            for (partition_num, force_track_partition) in force_track_node.tracked_partitions {
                for (db_sort_key, force_track_key) in force_track_partition.substates {
                    let tracked_node = self.tracked_nodes.get_mut(&node_id).unwrap();
                    let tracked_partition = tracked_node
                        .tracked_partitions
                        .get_mut(&partition_num)
                        .unwrap();
                    let tracked = &mut tracked_partition
                        .substates
                        .get_mut(&db_sort_key)
                        .unwrap()
                        .tracked;
                    *tracked = force_track_key.tracked;
                }
            }
        }
    }

    /// Finalizes changes captured by this substate store.
    ///
    ///  Returns the state changes and dependencies.
    pub fn finalize(
        self,
    ) -> (
        IndexMap<NodeId, TrackedNode>,
        IndexSet<(NodeId, PartitionNumber)>,
    ) {
        (self.tracked_nodes, self.deleted_partitions)
    }

    fn get_tracked_partition(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
    ) -> &mut TrackedPartition {
        self.tracked_nodes
            .entry(*node_id)
            .or_insert(TrackedNode::new(false))
            .tracked_partitions
            .entry(partition_num)
            .or_insert(TrackedPartition::new())
    }

    /// Returns tuple of TrackedKey and boolean value which is true if substate
    /// with specified db_key was found in tracked substates list (no db access needed).
    fn get_tracked_substate_virtualize<F: FnOnce() -> Option<IndexedScryptoValue>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        virtualize: F,
    ) -> (&mut TrackedKey, StoreAccessInfo) {
        let db_sort_key = M::to_db_sort_key(&substate_key);

        let partition = &mut self
            .tracked_nodes
            .entry(*node_id)
            .or_insert(TrackedNode::new(false))
            .tracked_partitions
            .entry(partition_num)
            .or_insert(TrackedPartition::new())
            .substates;
        let entry = partition.entry(db_sort_key.clone());

        let mut store_access = Vec::new();

        match entry {
            Entry::Vacant(e) => {
                let db_partition_key = M::to_db_partition_key(node_id, partition_num);
                let value = self
                    .substate_db
                    .get_substate(&db_partition_key, &db_sort_key)
                    .map(|e| IndexedScryptoValue::from_vec(e).expect("Failed to decode substate"));
                if let Some(value) = value {
                    store_access.push(StoreAccess::ReadFromDb(value.len()));
                    store_access.push(StoreAccess::WriteToTrack(value.len()));

                    let tracked = TrackedSubstateKey {
                        substate_key,
                        tracked: TrackedKey::ReadOnly(ReadOnly::Existent(RuntimeSubstate::new(
                            value,
                        ))),
                    };
                    e.insert(tracked);
                } else {
                    store_access.push(StoreAccess::ReadFromDbNotFound);

                    let value = virtualize();
                    if let Some(value) = value {
                        store_access.push(StoreAccess::WriteToTrack(value.len()));
                        let tracked = TrackedSubstateKey {
                            substate_key,
                            tracked: TrackedKey::ReadNonExistAndWrite(RuntimeSubstate::new(value)),
                        };
                        e.insert(tracked);
                    } else {
                        let tracked = TrackedSubstateKey {
                            substate_key,
                            tracked: TrackedKey::ReadOnly(ReadOnly::NonExistent),
                        };
                        e.insert(tracked);
                    }
                }
            }
            Entry::Occupied(mut entry) => {
                let read_only_non_existent = matches!(
                    entry.get().tracked,
                    TrackedKey::ReadOnly(ReadOnly::NonExistent)
                );
                if read_only_non_existent {
                    let value = virtualize();
                    if let Some(value) = value {
                        store_access.push(StoreAccess::WriteToTrack(value.len()));
                        let tracked = TrackedSubstateKey {
                            substate_key,
                            tracked: TrackedKey::ReadNonExistAndWrite(RuntimeSubstate::new(value)),
                        };
                        entry.insert(tracked);
                    } else {
                        let tracked = TrackedSubstateKey {
                            substate_key,
                            tracked: TrackedKey::ReadOnly(ReadOnly::NonExistent),
                        };
                        entry.insert(tracked);
                    }
                }
            }
        }

        (
            &mut partition.get_mut(&db_sort_key).unwrap().tracked,
            StoreAccessInfo::with_vector(store_access),
        )
    }

    fn get_tracked_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
    ) -> (&mut TrackedKey, StoreAccessInfo) {
        self.get_tracked_substate_virtualize(node_id, partition_num, substate_key, || None)
    }
}

impl<'s, S: SubstateDatabase, M: DatabaseKeyMapper> SubstateStore for Track<'s, S, M> {
    fn create_node(&mut self, node_id: NodeId, node_substates: NodeSubstates) -> StoreAccessInfo {
        let mut store_access = Vec::new();

        let tracked_partitions = node_substates
            .into_iter()
            .map(|(partition_num, partition)| {
                let partition_substates = partition
                    .into_iter()
                    .map(|(substate_key, value)| {
                        store_access.push(StoreAccess::WriteToTrack(value.len()));
                        let db_sort_key = M::to_db_sort_key(&substate_key);
                        let tracked = TrackedSubstateKey {
                            substate_key,
                            tracked: TrackedKey::New(RuntimeSubstate::new(value)),
                        };
                        (db_sort_key, tracked)
                    })
                    .collect();
                let tracked_partition = TrackedPartition::new_with_substates(partition_substates);
                (partition_num, tracked_partition)
            })
            .collect();

        self.tracked_nodes.insert(
            node_id,
            TrackedNode {
                tracked_partitions: tracked_partitions,
                is_new: true,
            },
        );

        StoreAccessInfo::with_vector(store_access)
    }

    fn set_substate(
        &mut self,
        node_id: NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    ) -> Result<StoreAccessInfo, SetSubstateError> {
        let db_sort_key = M::to_db_sort_key(&substate_key);

        let tracked_partition = self
            .tracked_nodes
            .entry(node_id)
            .or_insert(TrackedNode::new(false))
            .tracked_partitions
            .entry(partition_num)
            .or_insert(TrackedPartition::new());

        let entry = tracked_partition.substates.entry(db_sort_key);

        match entry {
            Entry::Vacant(e) => {
                let value_len = substate_value.len();

                let tracked = TrackedSubstateKey {
                    substate_key,
                    tracked: TrackedKey::WriteOnly(Write::Update(RuntimeSubstate::new(
                        substate_value,
                    ))),
                };
                e.insert(tracked);

                Ok(StoreAccessInfo::new()
                    .builder_push_if_not_empty(StoreAccess::WriteToTrack(value_len)))
            }
            Entry::Occupied(mut e) => {
                let tracked = e.get_mut();
                if let Some(runtime) = tracked.tracked.get_runtime_substate_mut() {
                    if runtime.lock_state.is_locked() {
                        return Err(SetSubstateError::SubstateLocked(
                            node_id,
                            partition_num,
                            substate_key,
                        ));
                    }
                }
                let new_value_len = substate_value.len();
                let old_value_len = if let Some(old_value) = tracked.tracked.get() {
                    old_value.len()
                } else {
                    0
                };

                tracked.tracked.set(substate_value);

                Ok(
                    StoreAccessInfo::new().builder_push_if_not_empty(StoreAccess::RewriteToTrack(
                        old_value_len,
                        new_value_len,
                    )),
                )
            }
        }
    }

    // Should not use on virtualized substates
    fn take_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(Option<IndexedScryptoValue>, StoreAccessInfo), TakeSubstateError> {
        let (tracked, mut store_access) =
            self.get_tracked_substate(node_id, partition_num, substate_key.clone());
        if let Some(runtime) = tracked.get_runtime_substate_mut() {
            if runtime.lock_state.is_locked() {
                return Err(TakeSubstateError::SubstateLocked(
                    *node_id,
                    partition_num,
                    substate_key.clone(),
                ));
            }
        }

        store_access.push(StoreAccess::DeleteFromTrack);

        Ok((tracked.take(), store_access))
    }

    fn scan_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> (Vec<IndexedScryptoValue>, StoreAccessInfo) {
        let count: usize = count.try_into().unwrap();
        let mut items = Vec::new();

        let node_updates = self.tracked_nodes.get(node_id);
        let is_new = node_updates
            .map(|tracked_node| tracked_node.is_new)
            .unwrap_or(false);
        let tracked_partition = node_updates.and_then(|n| n.tracked_partitions.get(&partition_num));

        let mut store_access = Vec::new();

        if let Some(tracked_partition) = tracked_partition {
            for tracked in tracked_partition.substates.values() {
                if items.len() == count {
                    return (items, StoreAccessInfo::with_vector(store_access));
                }

                // TODO: Check that substate is not write locked, before use outside of native blueprints
                if let Some(substate) = tracked.tracked.get() {
                    store_access.push(StoreAccess::ReadFromTrack(substate.len()));
                    items.push(substate.clone());
                }
            }
        }

        // Optimization, no need to go into database if the node is just created
        if is_new {
            return (items, StoreAccessInfo::with_vector(store_access));
        }

        let db_partition_key = M::to_db_partition_key(node_id, partition_num);
        let mut tracked_iter = TrackedIter::new(self.substate_db.list_entries(&db_partition_key));
        for (db_sort_key, substate) in &mut tracked_iter {
            if items.len() == count {
                break;
            }

            if tracked_partition
                .map(|tracked_partition| tracked_partition.substates.contains_key(&db_sort_key))
                .unwrap_or(false)
            {
                continue;
            }

            store_access.push(StoreAccess::ReadFromDb(substate.len()));
            items.push(IndexedScryptoValue::from_vec(substate).unwrap());
        }

        // Update track
        let num_iterations = tracked_iter.num_iterations;
        let tracked_partition = self.get_tracked_partition(node_id, partition_num);
        tracked_partition.range_read = u32::max(tracked_partition.range_read, num_iterations);

        (items, StoreAccessInfo::with_vector(store_access))
    }

    fn take_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> (Vec<IndexedScryptoValue>, StoreAccessInfo) {
        let count: usize = count.try_into().unwrap();
        let mut items = Vec::new();

        let node_updates = self.tracked_nodes.get_mut(node_id);
        let is_new = node_updates
            .as_ref()
            .map(|tracked_node| tracked_node.is_new)
            .unwrap_or(false);

        let mut store_access = Vec::new();
        // Check what we've currently got so far without going into database
        let mut tracked_partition =
            node_updates.and_then(|n| n.tracked_partitions.get_mut(&partition_num));
        if let Some(tracked_partition) = tracked_partition.as_mut() {
            for tracked in tracked_partition.substates.values_mut() {
                if items.len() == count {
                    return (items, StoreAccessInfo::with_vector(store_access));
                }

                // TODO: Check that substate is not locked, before use outside of native blueprints
                if let Some(value) = tracked.tracked.take() {
                    store_access.push(StoreAccess::ReadFromTrack(value.len()));
                    items.push(value);
                }
            }
        }

        // Optimization, no need to go into database if the node is just created
        if is_new {
            return (items, StoreAccessInfo::with_vector(store_access));
        }

        // Read from database
        let db_partition_key = M::to_db_partition_key(node_id, partition_num);
        let mut tracked_iter = TrackedIter::new(self.substate_db.list_entries(&db_partition_key));
        let new_updates = {
            let mut new_updates = Vec::new();
            for (db_sort_key, substate) in &mut tracked_iter {
                if items.len() == count {
                    break;
                }

                if tracked_partition
                    .as_ref()
                    .map(|tracked_partition| tracked_partition.substates.contains_key(&db_sort_key))
                    .unwrap_or(false)
                {
                    continue;
                }

                let value = IndexedScryptoValue::from_vec(substate).unwrap();

                // FIXME: review non-fungible implementation and see if this is an issue.
                // This only works because only NonFungible Vaults use this.
                // Will need to fix this by maintaining the invariant that the value
                // of the index contains the key. Or alternatively, change the abstraction
                // from being a Map to a Set
                let substate_key = SubstateKey::Map(value.as_slice().to_vec());

                let tracked = TrackedSubstateKey {
                    substate_key,
                    tracked: TrackedKey::ReadExistAndWrite(value.clone(), Write::Delete),
                };
                new_updates.push((db_sort_key, tracked));
                store_access.push(StoreAccess::ReadFromDb(value.len()));

                items.push(value);
            }
            new_updates
        };

        // Update track
        {
            let num_iterations = tracked_iter.num_iterations;
            let tracked_partition = self.get_tracked_partition(node_id, partition_num);
            tracked_partition.range_read = u32::max(tracked_partition.range_read, num_iterations);

            for (db_sort_key, tracked) in new_updates {
                let new_value_len = tracked.tracked.get().map(|value| value.len());

                if let Some(old_value) = tracked_partition.substates.insert(db_sort_key, tracked) {
                    if let Some(old_value) = old_value.tracked.get() {
                        if new_value_len.is_some() {
                            store_access.push(StoreAccess::RewriteToTrack(
                                old_value.len(),
                                new_value_len.unwrap(),
                            ));
                        }
                    }
                }
            }
        }

        (items, StoreAccessInfo::with_vector(store_access))
    }

    fn scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> (Vec<IndexedScryptoValue>, StoreAccessInfo) {
        // TODO: ensure we abort if any substates are write locked.
        let count: usize = count.try_into().unwrap();

        // initialize the track partition, since we will definitely need it: either to read values from it OR to update the `range_read` on it
        let tracked_node = self
            .tracked_nodes
            .entry(node_id.clone())
            .or_insert(TrackedNode::new(false));
        let tracked_partition = tracked_node
            .tracked_partitions
            .entry(partition_num)
            .or_insert(TrackedPartition::new());

        let mut store_access_db = Vec::new();
        // initialize the "from db" iterator: use `dyn`, since we want to skip it altogether if the node is marked as `is_new` in our track
        let mut db_values_count = 0u32;
        let raw_db_entries: Box<dyn Iterator<Item = PartitionEntry>> = if tracked_node.is_new {
            Box::new(empty()) // optimization: avoid touching the database altogether
        } else {
            let partition_key = M::to_db_partition_key(node_id, partition_num);
            Box::new(self.substate_db.list_entries(&partition_key))
        };
        let db_read_entries = raw_db_entries
            .map(|(key, db_value)| (key, IndexedScryptoValue::from_vec(db_value).unwrap()))
            .inspect(|(_key, value)| {
                db_values_count += 1;
                store_access_db.push(StoreAccess::ReadFromDb(value.len()));
            });

        let mut store_access_track = Vec::new();
        // initialize the "from track" iterator
        let tracked_entry_changes = tracked_partition
            .substates
            .iter()
            .map(|(key, tracked_substate_key)| {
                // TODO: ensure we abort if any substates are write locked.
                (key.clone(), tracked_substate_key.tracked.get().cloned())
            })
            .inspect(|(_key, value)| {
                if let Some(len) = value.as_ref().map(|value| value.len()) {
                    store_access_track.push(StoreAccess::ReadFromTrack(len));
                }
            });

        // construct the composite iterator, which applies changes read from our track on top of db values
        let items = OverlayingIterator::new(db_read_entries, tracked_entry_changes)
            .map(|(_key, value)| value)
            .take(count)
            .collect();

        store_access_db.extend(store_access_track);

        // Use the statistics (gathered by the `.inspect()`s above) to update the track's metadata and to return costing info
        tracked_partition.range_read = u32::max(tracked_partition.range_read, db_values_count);
        return (items, StoreAccessInfo::with_vector(store_access_db));
    }

    fn acquire_lock_virtualize<F: FnOnce() -> Option<IndexedScryptoValue>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        virtualize: F,
    ) -> Result<(u32, StoreAccessInfo), AcquireLockError> {
        // Load the substate from state track
        let (tracked, store_access) = self.get_tracked_substate_virtualize(
            node_id,
            partition_num,
            substate_key.clone(),
            virtualize,
        );

        // Check substate state
        if flags.contains(LockFlags::UNMODIFIED_BASE) {
            match tracked {
                TrackedKey::New(..) | TrackedKey::Garbage => {
                    return Err(AcquireLockError::LockUnmodifiedBaseOnNewSubstate(
                        *node_id,
                        partition_num,
                        substate_key.clone(),
                    ));
                }
                TrackedKey::WriteOnly(..)
                | TrackedKey::ReadExistAndWrite(..)
                | TrackedKey::ReadNonExistAndWrite(..) => {
                    return Err(AcquireLockError::LockUnmodifiedBaseOnOnUpdatedSubstate(
                        *node_id,
                        partition_num,
                        substate_key.clone(),
                    ));
                }
                TrackedKey::ReadOnly(..) => {
                    // Okay
                }
            }
        }

        let substate = match tracked.get_runtime_substate_mut() {
            Some(x) => x,
            None => {
                return Err(AcquireLockError::NotFound(
                    *node_id,
                    partition_num,
                    substate_key.clone(),
                ));
            }
        };

        // Check read/write permission
        substate.lock_state.try_lock(flags).map_err(|_| {
            AcquireLockError::SubstateLocked(*node_id, partition_num, substate_key.clone())
        })?;

        Ok((
            self.new_lock_handle(node_id, partition_num, substate_key, flags),
            store_access,
        ))
    }

    fn close_substate(&mut self, handle: u32) -> StoreAccessInfo {
        let (node_id, partition_num, substate_key, flags) =
            self.locks.remove(&handle).expect("Invalid lock handle");

        let (tracked, store_access) =
            self.get_tracked_substate(&node_id, partition_num, substate_key.clone());

        let substate = tracked
            .get_runtime_substate_mut()
            .expect("Could not have created lock on non-existent subsate");

        substate.lock_state.unlock();

        if flags.contains(LockFlags::FORCE_WRITE) {
            let db_sort_key = M::to_db_sort_key(&substate_key);
            let cloned_track = tracked.clone();

            self.force_write_tracked_nodes
                .entry(node_id)
                .or_insert(TrackedNode {
                    tracked_partitions: index_map_new(),
                    is_new: false,
                })
                .tracked_partitions
                .entry(partition_num)
                .or_insert(TrackedPartition::new())
                .substates
                .insert(
                    db_sort_key,
                    TrackedSubstateKey {
                        substate_key,
                        tracked: cloned_track,
                    },
                );
        }

        store_access
    }

    fn read_substate(&mut self, handle: u32) -> (&IndexedScryptoValue, StoreAccessInfo) {
        let (node_id, partition_num, substate_key, _flags) =
            self.locks.get(&handle).expect("Invalid lock handle");

        let node_id = *node_id;
        let partition_num = *partition_num;

        let (tracked, store_access) =
            self.get_tracked_substate(&node_id, partition_num, substate_key.clone());
        (
            tracked
                .get()
                .expect("Could not have created lock on non existent substate"),
            store_access,
        )
    }

    fn update_substate(
        &mut self,
        handle: u32,
        substate_value: IndexedScryptoValue,
    ) -> StoreAccessInfo {
        let (node_id, partition_num, substate_key, flags) =
            self.locks.get(&handle).expect("Invalid lock handle");

        if !flags.contains(LockFlags::MUTABLE) {
            panic!("No write permission for {}", handle);
        }

        let node_id = *node_id;
        let partition_num = *partition_num;

        let (tracked, mut store_access) =
            self.get_tracked_substate(&node_id, partition_num, substate_key.clone());

        match tracked {
            TrackedKey::New(substate)
            | TrackedKey::WriteOnly(Write::Update(substate))
            | TrackedKey::ReadExistAndWrite(_, Write::Update(substate))
            | TrackedKey::ReadNonExistAndWrite(substate) => {
                let size_old = substate.value.len();
                let size_new = substate_value.len();
                store_access.push_if_not_empty(StoreAccess::RewriteToTrack(size_old, size_new));

                substate.value = substate_value;
            }
            TrackedKey::ReadOnly(ReadOnly::NonExistent) => {
                let size = substate_value.len();
                store_access.push_if_not_empty(StoreAccess::WriteToTrack(size));

                let new_tracked =
                    TrackedKey::ReadNonExistAndWrite(RuntimeSubstate::new(substate_value));
                let mut old = mem::replace(tracked, new_tracked);
                tracked.get_runtime_substate_mut().unwrap().lock_state =
                    old.get_runtime_substate_mut().unwrap().lock_state;
            }
            TrackedKey::ReadOnly(ReadOnly::Existent(substate)) => {
                let size_old = substate.value.len();
                let size_new = substate_value.len();
                store_access.push_if_not_empty(StoreAccess::RewriteToTrack(size_old, size_new));

                let new_tracked = TrackedKey::ReadExistAndWrite(
                    substate.value.clone(),
                    Write::Update(RuntimeSubstate::new(substate_value)),
                );
                let mut old = mem::replace(tracked, new_tracked);
                tracked.get_runtime_substate_mut().unwrap().lock_state =
                    old.get_runtime_substate_mut().unwrap().lock_state;
            }
            TrackedKey::WriteOnly(Write::Delete)
            | TrackedKey::ReadExistAndWrite(_, Write::Delete)
            | TrackedKey::Garbage => {
                panic!("Could not have created lock on non existent substate")
            }
        };

        store_access
    }

    fn delete_partition(&mut self, node_id: &NodeId, partition_num: PartitionNumber) {
        self.deleted_partitions.insert((*node_id, partition_num));
    }
}
