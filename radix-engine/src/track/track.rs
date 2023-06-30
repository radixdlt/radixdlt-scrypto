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

use super::interface::{StoreCommit, StoreCommitInfo};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Default)]
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
pub struct TrackedSubstate {
    pub substate_key: SubstateKey,
    pub substate_value: TrackedSubstateValue,
}

// TODO: Add new virtualized
#[derive(Clone, Debug)]
pub enum TrackedSubstateValue {
    New(RuntimeSubstate),
    ReadOnly(ReadOnly),
    ReadExistAndWrite(IndexedScryptoValue, Write),
    ReadNonExistAndWrite(RuntimeSubstate),
    WriteOnly(Write),
    Garbage,
}

impl TrackedSubstateValue {
    fn get_runtime_substate_mut(&mut self) -> Option<&mut RuntimeSubstate> {
        match self {
            TrackedSubstateValue::New(substate)
            | TrackedSubstateValue::WriteOnly(Write::Update(substate))
            | TrackedSubstateValue::ReadOnly(ReadOnly::Existent(substate))
            | TrackedSubstateValue::ReadExistAndWrite(_, Write::Update(substate))
            | TrackedSubstateValue::ReadNonExistAndWrite(substate) => Some(substate),

            TrackedSubstateValue::WriteOnly(Write::Delete)
            | TrackedSubstateValue::ReadExistAndWrite(_, Write::Delete)
            | TrackedSubstateValue::ReadOnly(ReadOnly::NonExistent)
            | TrackedSubstateValue::Garbage => None,
        }
    }

    pub fn get(&self) -> Option<&IndexedScryptoValue> {
        match self {
            TrackedSubstateValue::New(substate)
            | TrackedSubstateValue::WriteOnly(Write::Update(substate))
            | TrackedSubstateValue::ReadOnly(ReadOnly::Existent(substate))
            | TrackedSubstateValue::ReadExistAndWrite(_, Write::Update(substate))
            | TrackedSubstateValue::ReadNonExistAndWrite(substate) => Some(&substate.value),
            TrackedSubstateValue::WriteOnly(Write::Delete)
            | TrackedSubstateValue::ReadExistAndWrite(_, Write::Delete)
            | TrackedSubstateValue::ReadOnly(ReadOnly::NonExistent)
            | TrackedSubstateValue::Garbage => None,
        }
    }

    pub fn set(&mut self, value: IndexedScryptoValue) {
        match self {
            TrackedSubstateValue::Garbage => {
                *self = TrackedSubstateValue::WriteOnly(Write::Update(RuntimeSubstate::new(value)));
            }
            TrackedSubstateValue::New(substate)
            | TrackedSubstateValue::WriteOnly(Write::Update(substate))
            | TrackedSubstateValue::ReadExistAndWrite(_, Write::Update(substate))
            | TrackedSubstateValue::ReadNonExistAndWrite(substate) => {
                substate.value = value;
            }
            TrackedSubstateValue::ReadOnly(ReadOnly::NonExistent) => {
                let new_tracked =
                    TrackedSubstateValue::ReadNonExistAndWrite(RuntimeSubstate::new(value));
                let mut old = mem::replace(self, new_tracked);
                self.get_runtime_substate_mut().unwrap().lock_state =
                    old.get_runtime_substate_mut().unwrap().lock_state;
            }
            TrackedSubstateValue::ReadOnly(ReadOnly::Existent(old)) => {
                let new_tracked = TrackedSubstateValue::ReadExistAndWrite(
                    old.value.clone(),
                    Write::Update(RuntimeSubstate::new(value)),
                );
                let mut old = mem::replace(self, new_tracked);
                self.get_runtime_substate_mut().unwrap().lock_state =
                    old.get_runtime_substate_mut().unwrap().lock_state;
            }
            TrackedSubstateValue::ReadExistAndWrite(_, write @ Write::Delete)
            | TrackedSubstateValue::WriteOnly(write @ Write::Delete) => {
                *write = Write::Update(RuntimeSubstate::new(value));
            }
        };
    }

    pub fn take(&mut self) -> Option<IndexedScryptoValue> {
        match self {
            TrackedSubstateValue::Garbage => None,
            TrackedSubstateValue::New(..) => {
                let old = mem::replace(self, TrackedSubstateValue::Garbage);
                old.into_value()
            }
            TrackedSubstateValue::WriteOnly(_) => {
                let old = mem::replace(self, TrackedSubstateValue::WriteOnly(Write::Delete));
                old.into_value()
            }
            TrackedSubstateValue::ReadExistAndWrite(_, write) => {
                let write = mem::replace(write, Write::Delete);
                write.into_value()
            }
            TrackedSubstateValue::ReadNonExistAndWrite(..) => {
                let old = mem::replace(self, TrackedSubstateValue::ReadOnly(ReadOnly::NonExistent));
                old.into_value()
            }
            TrackedSubstateValue::ReadOnly(ReadOnly::Existent(v)) => {
                let new_tracked =
                    TrackedSubstateValue::ReadExistAndWrite(v.value.clone(), Write::Delete);
                let old = mem::replace(self, new_tracked);
                old.into_value()
            }
            TrackedSubstateValue::ReadOnly(ReadOnly::NonExistent) => None,
        }
    }

    fn revert_writes(&mut self) {
        match self {
            TrackedSubstateValue::ReadOnly(..) | TrackedSubstateValue::Garbage => {}
            TrackedSubstateValue::New(..) | TrackedSubstateValue::WriteOnly(_) => {
                *self = TrackedSubstateValue::Garbage;
            }
            TrackedSubstateValue::ReadExistAndWrite(read, _) => {
                *self = TrackedSubstateValue::ReadOnly(ReadOnly::Existent(RuntimeSubstate::new(
                    read.clone(),
                )));
            }
            TrackedSubstateValue::ReadNonExistAndWrite(..) => {
                *self = TrackedSubstateValue::ReadOnly(ReadOnly::NonExistent);
            }
        }
    }

    pub fn into_value(self) -> Option<IndexedScryptoValue> {
        match self {
            TrackedSubstateValue::New(substate)
            | TrackedSubstateValue::WriteOnly(Write::Update(substate))
            | TrackedSubstateValue::ReadOnly(ReadOnly::Existent(substate))
            | TrackedSubstateValue::ReadNonExistAndWrite(substate)
            | TrackedSubstateValue::ReadExistAndWrite(_, Write::Update(substate)) => {
                Some(substate.value)
            }
            TrackedSubstateValue::WriteOnly(Write::Delete)
            | TrackedSubstateValue::ReadExistAndWrite(_, Write::Delete)
            | TrackedSubstateValue::ReadOnly(ReadOnly::NonExistent)
            | TrackedSubstateValue::Garbage => None,
        }
    }
}

#[derive(Debug)]
pub struct TrackedPartition {
    pub substates: BTreeMap<DbSortKey, TrackedSubstate>,
    pub range_read: u32,
}

impl TrackedPartition {
    pub fn new() -> Self {
        Self {
            substates: BTreeMap::new(),
            range_read: 0,
        }
    }

    pub fn new_with_substates(substates: BTreeMap<DbSortKey, TrackedSubstate>) -> Self {
        Self {
            substates,
            range_read: 0,
        }
    }

    pub fn revert_writes(&mut self) {
        for substate in &mut self.substates.values_mut() {
            substate.substate_value.revert_writes();
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
                let update = match tracked.substate_value {
                    TrackedSubstateValue::ReadOnly(..) | TrackedSubstateValue::Garbage => None,
                    TrackedSubstateValue::ReadNonExistAndWrite(substate)
                    | TrackedSubstateValue::New(substate) => {
                        Some(DatabaseUpdate::Set(substate.value.into()))
                    }
                    TrackedSubstateValue::ReadExistAndWrite(_, write)
                    | TrackedSubstateValue::WriteOnly(write) => match write {
                        Write::Delete => Some(DatabaseUpdate::Delete),
                        Write::Update(substate) => Some(DatabaseUpdate::Set(substate.value.into())),
                    },
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
    iter: Box<dyn Iterator<Item = (DbSortKey, IndexedScryptoValue)> + 'a>,
    num_iterations: u32,
}

impl<'a> TrackedIter<'a> {
    fn new(iter: Box<dyn Iterator<Item = (DbSortKey, IndexedScryptoValue)> + 'a>) -> Self {
        Self {
            iter,
            num_iterations: 0u32,
        }
    }
}

impl<'a> Iterator for TrackedIter<'a> {
    type Item = (DbSortKey, IndexedScryptoValue);

    fn next(&mut self) -> Option<Self::Item> {
        self.num_iterations = self.num_iterations + 1;
        self.iter.next()
    }
}

/// Transaction-wide states and side effects
pub struct Track<'s, S: SubstateDatabase, M: DatabaseKeyMapper> {
    /// Substate database, use `get_substate_from_db` and `list_entries_from_db` for access
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

    fn get_substate_from_db(
        substate_db: &'s S,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
        store_access: &mut StoreAccessInfo,
    ) -> Option<IndexedScryptoValue> {
        let result = substate_db
            .get_substate(partition_key, sort_key)
            .map(|e| IndexedScryptoValue::from_vec(e).expect("Failed to decode substate"));
        if let Some(x) = &result {
            store_access.push(StoreAccess::ReadFromDb(x.len()));
        } else {
            store_access.push(StoreAccess::ReadFromDbNotFound);
        }
        result
    }

    fn list_entries_from_db<'x>(
        substate_db: &'x S,
        partition_key: &DbPartitionKey,
        store_access: &'x mut StoreAccessInfo,
    ) -> Box<dyn Iterator<Item = (DbSortKey, IndexedScryptoValue)> + 'x> {
        struct TracedIterator<'a, 'b> {
            iterator: Box<dyn Iterator<Item = PartitionEntry> + 'a>,
            store_access: &'b mut StoreAccessInfo,
        }

        impl<'a, 'b> Iterator for TracedIterator<'a, 'b> {
            type Item = (DbSortKey, IndexedScryptoValue);

            fn next(&mut self) -> Option<Self::Item> {
                let result = self.iterator.next();
                if let Some(x) = result {
                    self.store_access.push(StoreAccess::ReadFromDb(x.1.len()));
                    Some((
                        x.0,
                        IndexedScryptoValue::from_vec(x.1).expect("Failed to decode substate"),
                    ))
                } else {
                    self.store_access.push(StoreAccess::ReadFromDbNotFound);
                    None
                }
            }
        }

        Box::new(TracedIterator {
            iterator: substate_db.list_entries(partition_key),
            store_access,
        })
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
                        .substate_value;
                    *tracked = force_track_key.substate_value;
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

    /// Returns tuple of TrackedSubstateValue and boolean value which is true if substate
    /// with specified db_key was found in tracked substates list (no db access needed).
    fn get_tracked_substate_virtualize<F: FnOnce() -> Option<IndexedScryptoValue>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        virtualize: F,
        store_access: &mut StoreAccessInfo,
    ) -> &mut TrackedSubstateValue {
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

        match entry {
            Entry::Vacant(e) => {
                let db_partition_key = M::to_db_partition_key(node_id, partition_num);
                let value = Self::get_substate_from_db(
                    self.substate_db,
                    &db_partition_key,
                    &db_sort_key,
                    store_access,
                );
                if let Some(value) = value {
                    store_access.push(StoreAccess::ReadFromDb(value.len()));
                    store_access.push(StoreAccess::NewEntryInTrack);

                    let tracked = TrackedSubstate {
                        substate_key,
                        substate_value: TrackedSubstateValue::ReadOnly(ReadOnly::Existent(
                            RuntimeSubstate::new(value),
                        )),
                    };
                    e.insert(tracked);
                } else {
                    store_access.push(StoreAccess::ReadFromDbNotFound);
                    store_access.push(StoreAccess::NewEntryInTrack);

                    let value = virtualize();
                    if let Some(value) = value {
                        let tracked = TrackedSubstate {
                            substate_key,
                            substate_value: TrackedSubstateValue::ReadNonExistAndWrite(
                                RuntimeSubstate::new(value),
                            ),
                        };
                        e.insert(tracked);
                    } else {
                        let tracked = TrackedSubstate {
                            substate_key,
                            substate_value: TrackedSubstateValue::ReadOnly(ReadOnly::NonExistent),
                        };
                        e.insert(tracked);
                    }
                }
            }
            Entry::Occupied(mut entry) => {
                let read_only_non_existent = matches!(
                    entry.get().substate_value,
                    TrackedSubstateValue::ReadOnly(ReadOnly::NonExistent)
                );
                if read_only_non_existent {
                    let value = virtualize();
                    if let Some(value) = value {
                        let tracked = TrackedSubstate {
                            substate_key,
                            substate_value: TrackedSubstateValue::ReadNonExistAndWrite(
                                RuntimeSubstate::new(value),
                            ),
                        };
                        entry.insert(tracked);
                    } else {
                        let tracked = TrackedSubstate {
                            substate_key,
                            substate_value: TrackedSubstateValue::ReadOnly(ReadOnly::NonExistent),
                        };
                        entry.insert(tracked);
                    }
                }
            }
        }

        &mut partition.get_mut(&db_sort_key).unwrap().substate_value
    }

    fn get_tracked_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        store_access: &mut StoreAccessInfo,
    ) -> &mut TrackedSubstateValue {
        self.get_tracked_substate_virtualize(
            node_id,
            partition_num,
            substate_key,
            || None,
            store_access,
        )
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
                        store_access.push(StoreAccess::NewEntryInTrack);
                        let db_sort_key = M::to_db_sort_key(&substate_key);
                        let tracked = TrackedSubstate {
                            substate_key,
                            substate_value: TrackedSubstateValue::New(RuntimeSubstate::new(value)),
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

        store_access
    }

    fn set_substate(
        &mut self,
        node_id: NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    ) -> Result<StoreAccessInfo, SetSubstateError> {
        let mut store_access = Vec::new();

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
                store_access.push(StoreAccess::NewEntryInTrack);
                let tracked = TrackedSubstate {
                    substate_key,
                    substate_value: TrackedSubstateValue::WriteOnly(Write::Update(
                        RuntimeSubstate::new(substate_value),
                    )),
                };
                e.insert(tracked);
            }
            Entry::Occupied(mut e) => {
                let tracked = e.get_mut();
                if let Some(runtime) = tracked.substate_value.get_runtime_substate_mut() {
                    if runtime.lock_state.is_locked() {
                        return Err(SetSubstateError::SubstateLocked(
                            node_id,
                            partition_num,
                            substate_key,
                        ));
                    }
                }

                tracked.substate_value.set(substate_value);
            }
        }

        Ok(store_access)
    }

    // Should not use on virtualized substates
    fn take_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(Option<IndexedScryptoValue>, StoreAccessInfo), TakeSubstateError> {
        let mut store_access = Vec::new();

        let tracked = self.get_tracked_substate(
            node_id,
            partition_num,
            substate_key.clone(),
            &mut store_access,
        );
        if let Some(runtime) = tracked.get_runtime_substate_mut() {
            if runtime.lock_state.is_locked() {
                return Err(TakeSubstateError::SubstateLocked(
                    *node_id,
                    partition_num,
                    substate_key.clone(),
                ));
            }
        }

        let value = tracked.take();

        Ok((value, store_access))
    }

    fn scan_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> (Vec<IndexedScryptoValue>, StoreAccessInfo) {
        let mut store_access = Vec::new();

        let count: usize = count.try_into().unwrap();
        let mut items = Vec::new();

        let node_updates = self.tracked_nodes.get(node_id);
        let is_new = node_updates
            .map(|tracked_node| tracked_node.is_new)
            .unwrap_or(false);
        let tracked_partition = node_updates.and_then(|n| n.tracked_partitions.get(&partition_num));

        if let Some(tracked_partition) = tracked_partition {
            for tracked in tracked_partition.substates.values() {
                if items.len() == count {
                    return (items, store_access);
                }

                // TODO: Check that substate is not write locked, before use outside of native blueprints
                if let Some(substate) = tracked.substate_value.get() {
                    items.push(substate.clone());
                }
            }
        }

        // Optimization, no need to go into database if the node is just created
        if is_new {
            return (items, store_access);
        }

        let db_partition_key = M::to_db_partition_key(node_id, partition_num);
        let mut tracked_iter = TrackedIter::new(Self::list_entries_from_db(
            self.substate_db,
            &db_partition_key,
            &mut store_access,
        ));
        for (db_sort_key, value) in &mut tracked_iter {
            if items.len() == count {
                break;
            }

            if tracked_partition
                .map(|tracked_partition| tracked_partition.substates.contains_key(&db_sort_key))
                .unwrap_or(false)
            {
                continue;
            }

            items.push(value);
        }

        // Update track
        let num_iterations = tracked_iter.num_iterations;
        let tracked_partition = self.get_tracked_partition(node_id, partition_num);
        tracked_partition.range_read = u32::max(tracked_partition.range_read, num_iterations);

        drop(tracked_iter);
        (items, store_access)
    }

    fn take_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> (Vec<IndexedScryptoValue>, StoreAccessInfo) {
        let mut store_access = Vec::new();

        let count: usize = count.try_into().unwrap();
        let mut items = Vec::new();

        let node_updates = self.tracked_nodes.get_mut(node_id);
        let is_new = node_updates
            .as_ref()
            .map(|tracked_node| tracked_node.is_new)
            .unwrap_or(false);

        // Check what we've currently got so far without going into database
        let mut tracked_partition =
            node_updates.and_then(|n| n.tracked_partitions.get_mut(&partition_num));
        if let Some(tracked_partition) = tracked_partition.as_mut() {
            for tracked in tracked_partition.substates.values_mut() {
                if items.len() == count {
                    return (items, store_access);
                }

                // TODO: Check that substate is not locked, before use outside of native blueprints
                if let Some(value) = tracked.substate_value.take() {
                    items.push(value);
                }
            }
        }

        // Optimization, no need to go into database if the node is just created
        if is_new {
            return (items, store_access);
        }

        // Read from database
        let db_partition_key = M::to_db_partition_key(node_id, partition_num);
        let mut tracked_iter = TrackedIter::new(Self::list_entries_from_db(
            self.substate_db,
            &db_partition_key,
            &mut store_access,
        ));
        let new_updates = {
            let mut new_updates = Vec::new();
            for (db_sort_key, value) in &mut tracked_iter {
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

                // FIXME: review non-fungible implementation and see if this is an issue.
                // This only works because only NonFungible Vaults use this.
                // Will need to fix this by maintaining the invariant that the value
                // of the index contains the key. Or alternatively, change the abstraction
                // from being a Map to a Set
                let substate_key = SubstateKey::Map(value.as_slice().to_vec());

                let tracked = TrackedSubstate {
                    substate_key,
                    substate_value: TrackedSubstateValue::ReadExistAndWrite(
                        value.clone(),
                        Write::Delete,
                    ),
                };
                new_updates.push((db_sort_key, tracked));

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
                tracked_partition.substates.insert(db_sort_key, tracked);
            }
        }

        drop(tracked_iter);
        (items, store_access)
    }

    fn scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> (Vec<IndexedScryptoValue>, StoreAccessInfo) {
        let mut store_access = Vec::new();

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

        // initialize the "from db" iterator: use `dyn`, since we want to skip it altogether if the node is marked as `is_new` in our track
        let mut db_values_count = 0u32;
        let raw_db_entries: Box<dyn Iterator<Item = (DbSortKey, IndexedScryptoValue)>> =
            if tracked_node.is_new {
                Box::new(empty()) // optimization: avoid touching the database altogether
            } else {
                let partition_key = M::to_db_partition_key(node_id, partition_num);
                Box::new(Self::list_entries_from_db(
                    self.substate_db,
                    &partition_key,
                    &mut store_access,
                ))
            };
        let db_read_entries = raw_db_entries.inspect(|(_key, _value)| {
            db_values_count += 1;
        });

        // initialize the "from track" iterator
        let tracked_entry_changes =
            tracked_partition
                .substates
                .iter()
                .map(|(key, tracked_substate)| {
                    // TODO: ensure we abort if any substates are write locked.
                    (key.clone(), tracked_substate.substate_value.get().cloned())
                });

        // construct the composite iterator, which applies changes read from our track on top of db values
        let items = OverlayingIterator::new(db_read_entries, tracked_entry_changes)
            .map(|(_key, value)| value)
            .take(count)
            .collect();

        // Use the statistics (gathered by the `.inspect()`s above) to update the track's metadata and to return costing info
        tracked_partition.range_read = u32::max(tracked_partition.range_read, db_values_count);

        return (items, store_access);
    }

    fn acquire_lock_virtualize<F: FnOnce() -> Option<IndexedScryptoValue>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        virtualize: F,
    ) -> Result<(u32, StoreAccessInfo), AcquireLockError> {
        let mut store_access = Vec::new();

        // Load the substate from state track
        let tracked = self.get_tracked_substate_virtualize(
            node_id,
            partition_num,
            substate_key.clone(),
            virtualize,
            &mut store_access,
        );

        // Check substate state
        if flags.contains(LockFlags::UNMODIFIED_BASE) {
            match tracked {
                TrackedSubstateValue::New(..) | TrackedSubstateValue::Garbage => {
                    return Err(AcquireLockError::LockUnmodifiedBaseOnNewSubstate(
                        *node_id,
                        partition_num,
                        substate_key.clone(),
                    ));
                }
                TrackedSubstateValue::WriteOnly(..)
                | TrackedSubstateValue::ReadExistAndWrite(..)
                | TrackedSubstateValue::ReadNonExistAndWrite(..) => {
                    return Err(AcquireLockError::LockUnmodifiedBaseOnOnUpdatedSubstate(
                        *node_id,
                        partition_num,
                        substate_key.clone(),
                    ));
                }
                TrackedSubstateValue::ReadOnly(..) => {
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

        let handle = self.new_lock_handle(node_id, partition_num, substate_key, flags);

        Ok((handle, store_access))
    }

    fn close_substate(&mut self, handle: u32) -> StoreAccessInfo {
        let mut store_access = Vec::new();

        let (node_id, partition_num, substate_key, flags) =
            self.locks.remove(&handle).expect("Invalid lock handle");

        let tracked = self.get_tracked_substate(
            &node_id,
            partition_num,
            substate_key.clone(),
            &mut store_access,
        );

        let substate = tracked
            .get_runtime_substate_mut()
            .expect("Could not have created lock on non-existent substate");

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
                    TrackedSubstate {
                        substate_key,
                        substate_value: cloned_track,
                    },
                );
        }

        store_access
    }

    fn read_substate(&mut self, handle: u32) -> (&IndexedScryptoValue, StoreAccessInfo) {
        let mut store_access = Vec::new();

        // Sanity check flag
        let (node_id, partition_num, substate_key, _flags) = self
            .locks
            .get(&handle)
            .cloned()
            .expect("Invalid lock handle");

        // Read substate
        let tracked = self.get_tracked_substate(
            &node_id,
            partition_num,
            substate_key.clone(),
            &mut store_access,
        );
        let value = tracked
            .get()
            .expect("Could not have created lock on non existent substate");

        (value, store_access)
    }

    fn update_substate(
        &mut self,
        handle: u32,
        substate_value: IndexedScryptoValue,
    ) -> StoreAccessInfo {
        let mut store_access = Vec::new();

        // Sanity check flag
        let (node_id, partition_num, substate_key, flags) = self
            .locks
            .get(&handle)
            .cloned()
            .expect("Invalid lock handle");
        if !flags.contains(LockFlags::MUTABLE) {
            panic!("No write permission for {}", handle);
        }

        // Update substate
        let tracked = self.get_tracked_substate(
            &node_id,
            partition_num,
            substate_key.clone(),
            &mut store_access,
        );
        match tracked {
            TrackedSubstateValue::New(substate)
            | TrackedSubstateValue::WriteOnly(Write::Update(substate))
            | TrackedSubstateValue::ReadExistAndWrite(_, Write::Update(substate))
            | TrackedSubstateValue::ReadNonExistAndWrite(substate) => {
                substate.value = substate_value;
            }
            TrackedSubstateValue::ReadOnly(ReadOnly::NonExistent) => {
                let new_tracked = TrackedSubstateValue::ReadNonExistAndWrite(RuntimeSubstate::new(
                    substate_value,
                ));
                let mut old = mem::replace(tracked, new_tracked);
                tracked.get_runtime_substate_mut().unwrap().lock_state =
                    old.get_runtime_substate_mut().unwrap().lock_state;
            }
            TrackedSubstateValue::ReadOnly(ReadOnly::Existent(substate)) => {
                let new_tracked = TrackedSubstateValue::ReadExistAndWrite(
                    substate.value.clone(),
                    Write::Update(RuntimeSubstate::new(substate_value)),
                );
                let mut old = mem::replace(tracked, new_tracked);
                tracked.get_runtime_substate_mut().unwrap().lock_state =
                    old.get_runtime_substate_mut().unwrap().lock_state;
            }
            TrackedSubstateValue::WriteOnly(Write::Delete)
            | TrackedSubstateValue::ReadExistAndWrite(_, Write::Delete)
            | TrackedSubstateValue::Garbage => {
                panic!("Could not have created lock on non existent substate")
            }
        };

        store_access
    }

    fn delete_partition(&mut self, node_id: &NodeId, partition_num: PartitionNumber) {
        // This is used for transaction tracker only, for which we don't account for store access.

        self.deleted_partitions.insert((*node_id, partition_num));
    }

    fn get_commit_info(&mut self) -> StoreCommitInfo {
        let mut store_commit = Vec::new();

        for (node_id, node) in &self.tracked_nodes {
            for (partition_number, partition) in &node.tracked_partitions {
                for (sort_key, substate) in &partition.substates {
                    match &substate.substate_value {
                        TrackedSubstateValue::New(v) => {
                            store_commit.push(StoreCommit::Insert {
                                node_id: node_id.clone(),
                                size: v.value.len(),
                            });
                        }
                        TrackedSubstateValue::ReadOnly(_) => {
                            // No op
                        }
                        TrackedSubstateValue::ReadExistAndWrite(old_value, write) => match write {
                            Write::Update(x) => {
                                store_commit.push(StoreCommit::Update {
                                    node_id: node_id.clone(),
                                    size: x.value.len(),
                                    old_size: old_value.len(),
                                });
                            }
                            Write::Delete => {
                                store_commit.push(StoreCommit::Delete {
                                    node_id: node_id.clone(),
                                    old_size: old_value.len(),
                                });
                            }
                        },
                        TrackedSubstateValue::ReadNonExistAndWrite(value) => {
                            store_commit.push(StoreCommit::Insert {
                                node_id: node_id.clone(),
                                size: value.value.len(),
                            });
                        }
                        TrackedSubstateValue::WriteOnly(write) => {
                            let old_size = self
                                .substate_db
                                .get_substate(
                                    &M::to_db_partition_key(node_id, *partition_number),
                                    &sort_key,
                                )
                                .map(|x| x.len());

                            match (old_size, write) {
                                (Some(old_size), Write::Update(x)) => {
                                    store_commit.push(StoreCommit::Update {
                                        node_id: node_id.clone(),
                                        size: x.value.len(),
                                        old_size,
                                    });
                                }
                                (Some(old_size), Write::Delete) => {
                                    store_commit.push(StoreCommit::Delete {
                                        node_id: node_id.clone(),
                                        old_size,
                                    });
                                }
                                (None, Write::Update(x)) => {
                                    store_commit.push(StoreCommit::Insert {
                                        node_id: node_id.clone(),
                                        size: x.value.len(),
                                    });
                                }
                                (None, Write::Delete) => {
                                    // TODO: this should never happen?
                                }
                            }
                        }
                        TrackedSubstateValue::Garbage => {
                            // No op
                        }
                    }
                }
            }
        }

        store_commit
    }
}
