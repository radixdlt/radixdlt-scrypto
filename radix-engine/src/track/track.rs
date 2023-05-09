use crate::track::db_key_mapper::DatabaseKeyMapper;
use crate::track::interface::{
    AcquireLockError, NodeSubstates, SetSubstateError, SubstateStore, SubstateStoreDbAccessInfo,
    TakeSubstateError,
};
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::types::*;
use radix_engine_store_interface::interface::{
    DatabaseUpdate, DatabaseUpdates, DbSortKey, PartitionEntry, SubstateDatabase,
};
use sbor::rust::collections::btree_map::Entry;
use sbor::rust::mem;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct StateUpdates {
    pub database_updates: DatabaseUpdates,
    pub system_updates: SystemUpdates,
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
    pub range_read: Option<u32>,
}

impl TrackedPartition {
    pub fn new() -> Self {
        Self {
            substates: BTreeMap::new(),
            range_read: None,
        }
    }

    pub fn new_with_substates(substates: BTreeMap<DbSortKey, TrackedSubstateKey>) -> Self {
        Self {
            substates,
            range_read: None,
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

    StateUpdates {
        database_updates,
        system_updates,
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
    pub fn finalize(self) -> IndexMap<NodeId, TrackedNode> {
        self.tracked_nodes
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
            .or_insert(TrackedPartition::new());

        self.tracked_nodes
            .get_mut(node_id)
            .unwrap()
            .tracked_partitions
            .get_mut(&partition_num)
            .unwrap()
    }

    /// Returns tuple of TrackedKey and boolean value which is true if substate
    /// with specified db_key was found in tracked substates list (no db access needed).
    fn get_tracked_substate_virtualize<F: FnOnce() -> Option<IndexedScryptoValue>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        virtualize: F,
    ) -> (&mut TrackedKey, bool) {
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

        let found = match entry {
            Entry::Vacant(e) => {
                let db_partition_key = M::to_db_partition_key(node_id, partition_num);
                let value = self
                    .substate_db
                    .get_substate(&db_partition_key, &db_sort_key)
                    .map(|e| IndexedScryptoValue::from_vec(e).expect("Failed to decode substate"));
                if let Some(value) = value {
                    let tracked = TrackedSubstateKey {
                        substate_key,
                        tracked: TrackedKey::ReadOnly(ReadOnly::Existent(RuntimeSubstate::new(
                            value,
                        ))),
                    };
                    e.insert(tracked);
                } else {
                    let value = virtualize();
                    if let Some(value) = value {
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
                false
            }
            Entry::Occupied(..) => true,
        };

        (&mut partition.get_mut(&db_sort_key).unwrap().tracked, found)
    }

    fn get_tracked_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
    ) -> (&mut TrackedKey, bool) {
        self.get_tracked_substate_virtualize(node_id, partition_num, substate_key, || None)
    }
}

impl<'s, S: SubstateDatabase, M: DatabaseKeyMapper> SubstateStore for Track<'s, S, M> {
    fn create_node(&mut self, node_id: NodeId, node_substates: NodeSubstates) {
        let tracked_partitions = node_substates
            .into_iter()
            .map(|(partition_num, partition)| {
                let partition_substates = partition
                    .into_iter()
                    .map(|(substate_key, value)| {
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
    }

    fn set_substate(
        &mut self,
        node_id: NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    ) -> Result<(), SetSubstateError> {
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
                let tracked = TrackedSubstateKey {
                    substate_key,
                    tracked: TrackedKey::WriteOnly(Write::Update(RuntimeSubstate::new(
                        substate_value,
                    ))),
                };
                e.insert(tracked);
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

                tracked.tracked.set(substate_value);
            }
        }

        Ok(())
    }

    // Should not use on virtualized substates
    fn take_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<(Option<IndexedScryptoValue>, SubstateStoreDbAccessInfo), TakeSubstateError> {
        let (tracked, found_in_tracked_list) =
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

        let store_access = if found_in_tracked_list {
            SubstateStoreDbAccessInfo::NoAccess
        } else {
            SubstateStoreDbAccessInfo::Take
        };

        Ok((tracked.take(), store_access))
    }

    fn scan_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> (Vec<IndexedScryptoValue>, SubstateStoreDbAccessInfo) {
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
                    return (
                        items,
                        SubstateStoreDbAccessInfo::Scan {
                            first_time_read_records_count: 0,
                            read_from_track_count: count,
                        },
                    );
                }

                // TODO: Check that substate is not write locked
                if let Some(substate) = tracked.tracked.get() {
                    items.push(substate.clone());
                }
            }
        }

        let items_intermediate_len = items.len();

        // Optimization, no need to go into database if the node is just created
        if is_new {
            return (
                items,
                SubstateStoreDbAccessInfo::Scan {
                    first_time_read_records_count: 0,
                    read_from_track_count: items_intermediate_len,
                },
            );
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

            items.push(IndexedScryptoValue::from_vec(substate).unwrap());
        }

        // Update track
        let num_iterations = tracked_iter.num_iterations;
        let tracked_partition = self.get_tracked_partition(node_id, partition_num);
        let next_range_read = tracked_partition
            .range_read
            .map(|cur| u32::max(cur, num_iterations))
            .unwrap_or(num_iterations);
        tracked_partition.range_read = Some(next_range_read);

        let first_time_read_records_count = items.len() - items_intermediate_len;

        (
            items,
            SubstateStoreDbAccessInfo::Scan {
                first_time_read_records_count,
                read_from_track_count: items_intermediate_len,
            },
        )
    }

    fn take_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> (Vec<IndexedScryptoValue>, SubstateStoreDbAccessInfo) {
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
                    return (
                        items,
                        SubstateStoreDbAccessInfo::TakeMany {
                            first_time_read_records_count: 0,
                            read_from_track_count: count,
                            update_count: 0,
                        },
                    );
                }

                // TODO: Check that substate is not locked
                if let Some(value) = tracked.tracked.take() {
                    items.push(value);
                }
            }
        }

        let items_intermediate_len = items.len();

        // Optimization, no need to go into database if the node is just created
        if is_new {
            return (
                items,
                SubstateStoreDbAccessInfo::TakeMany {
                    first_time_read_records_count: 0,
                    read_from_track_count: items_intermediate_len,
                    update_count: 0,
                },
            );
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

                // FIXME: This only works because only NonFungible Vaults use this.
                // FIXME: Will need to fix this by maintaining the invariant that the value
                // FIXME: of the index contains the key. Or alternatively, change the abstraction
                // FIXME: from being a Map to a Set
                let substate_key = SubstateKey::Map(value.as_slice().to_vec());

                let tracked = TrackedSubstateKey {
                    substate_key,
                    tracked: TrackedKey::ReadExistAndWrite(value.clone(), Write::Delete),
                };
                new_updates.push((db_sort_key, tracked));
                items.push(value);
            }
            new_updates
        };

        // Update track
        let update_count = {
            let num_iterations = tracked_iter.num_iterations;
            let tracked_partition = self.get_tracked_partition(node_id, partition_num);
            let next_range_read = tracked_partition
                .range_read
                .map(|cur| u32::max(cur, num_iterations))
                .unwrap_or(num_iterations);
            tracked_partition.range_read = Some(next_range_read);
            let update_count = new_updates.len();
            for (db_sort_key, tracked) in new_updates {
                tracked_partition.substates.insert(db_sort_key, tracked);
            }
            update_count
        };

        let first_time_read_records_count = items.len() - items_intermediate_len;

        (
            items,
            SubstateStoreDbAccessInfo::TakeMany {
                first_time_read_records_count,
                read_from_track_count: items_intermediate_len,
                update_count,
            },
        )
    }

    fn scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> (Vec<IndexedScryptoValue>, SubstateStoreDbAccessInfo) {
        // TODO: Add partition dependencies/lock
        let count: usize = count.try_into().unwrap();
        let node_updates = self.tracked_nodes.get_mut(node_id);
        let is_new = node_updates
            .as_ref()
            .map(|tracked_node| tracked_node.is_new)
            .unwrap_or(false);
        let tracked_partition = node_updates.and_then(|n| n.tracked_partitions.get(&partition_num));

        if is_new {
            let mut items = Vec::new();
            if let Some(tracked_partition) = tracked_partition {
                for (_key, tracked) in tracked_partition.substates.iter() {
                    if items.len() == count {
                        break;
                    }

                    // TODO: Check that substate is not write locked
                    if let Some(substate) = tracked.tracked.get() {
                        items.push(substate.clone());
                    }
                }
            }

            let read_from_track_count = items.len();

            return (
                items,
                SubstateStoreDbAccessInfo::ScanSorted {
                    first_time_read_records_count: 0,
                    read_from_track_count,
                },
            );
        }

        // TODO: Add interleaving updates
        let db_partition_key = M::to_db_partition_key(node_id, partition_num);
        let tracked_iter = TrackedIter::new(self.substate_db.list_entries(&db_partition_key));
        let items: Vec<IndexedScryptoValue> = tracked_iter
            .take(count)
            .map(|(_key, buf)| IndexedScryptoValue::from_vec(buf).unwrap())
            .collect();

        // Update track
        {
            let num_iterations: u32 = items.len().try_into().unwrap();
            let tracked_partition = self.get_tracked_partition(node_id, partition_num);
            let next_range_read = tracked_partition
                .range_read
                .map(|cur| u32::max(cur, num_iterations))
                .unwrap_or(num_iterations);
            tracked_partition.range_read = Some(next_range_read);
        }

        let first_time_read_records_count = items.len();

        (
            items,
            SubstateStoreDbAccessInfo::ScanSorted {
                first_time_read_records_count,
                read_from_track_count: 0,
            },
        )
    }

    fn acquire_lock_virtualize<F: FnOnce() -> Option<IndexedScryptoValue>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        virtualize: F,
    ) -> Result<(u32, SubstateStoreDbAccessInfo), AcquireLockError> {
        // Load the substate from state track
        let (tracked, found_in_tracked_list) = self.get_tracked_substate_virtualize(
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

        let substate = tracked
            .get_runtime_substate_mut()
            .ok_or(AcquireLockError::NotFound(
                *node_id,
                partition_num,
                substate_key.clone(),
            ))?;

        // Check read/write permission
        substate.lock_state.try_lock(flags).map_err(|_| {
            AcquireLockError::SubstateLocked(*node_id, partition_num, substate_key.clone())
        })?;

        let store_access = if found_in_tracked_list {
            SubstateStoreDbAccessInfo::NoAccess
        } else {
            SubstateStoreDbAccessInfo::AcquireLock
        };

        Ok((
            self.new_lock_handle(node_id, partition_num, substate_key, flags),
            store_access,
        ))
    }

    fn release_lock(&mut self, handle: u32) {
        let (node_id, partition_num, substate_key, flags) =
            self.locks.remove(&handle).expect("Invalid lock handle");

        let (tracked, _) = self.get_tracked_substate(&node_id, partition_num, substate_key.clone());

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
    }

    fn read_substate(&mut self, handle: u32) -> &IndexedScryptoValue {
        let (node_id, partition_num, substate_key, _flags) =
            self.locks.get(&handle).expect("Invalid lock handle");

        let node_id = *node_id;
        let partition_num = *partition_num;

        let (tracked, _) = self.get_tracked_substate(&node_id, partition_num, substate_key.clone());
        tracked
            .get()
            .expect("Could not have created lock on non existent substate")
    }

    fn update_substate(&mut self, handle: u32, substate_value: IndexedScryptoValue) {
        let (node_id, partition_num, substate_key, flags) =
            self.locks.get(&handle).expect("Invalid lock handle");

        if !flags.contains(LockFlags::MUTABLE) {
            panic!("No write permission for {}", handle);
        }

        let node_id = *node_id;
        let partition_num = *partition_num;

        let (tracked, _) = self.get_tracked_substate(&node_id, partition_num, substate_key.clone());

        match tracked {
            TrackedKey::New(substate)
            | TrackedKey::WriteOnly(Write::Update(substate))
            | TrackedKey::ReadExistAndWrite(_, Write::Update(substate))
            | TrackedKey::ReadNonExistAndWrite(substate) => {
                substate.value = substate_value;
            }
            TrackedKey::ReadOnly(ReadOnly::NonExistent) => {
                let new_tracked =
                    TrackedKey::ReadNonExistAndWrite(RuntimeSubstate::new(substate_value));
                let mut old = mem::replace(tracked, new_tracked);
                tracked.get_runtime_substate_mut().unwrap().lock_state =
                    old.get_runtime_substate_mut().unwrap().lock_state;
            }
            TrackedKey::ReadOnly(ReadOnly::Existent(substate)) => {
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
    }
}
