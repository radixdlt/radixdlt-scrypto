use crate::internal_prelude::*;
use crate::track::LegacyStateUpdates;
use radix_substate_store_interface::interface::{
    DatabaseUpdates, DbSubstateValue, NodeDatabaseUpdates, PartitionDatabaseUpdates,
};
use radix_substate_store_interface::{
    db_key_mapper::DatabaseKeyMapper,
    interface::{DatabaseUpdate, DbSortKey},
};
use sbor::rust::{cmp::*, iter::*, mem};

use super::TrackedSubstates;

/// A tree-like description of all updates that happened to a stored state, to be included as a part
/// of a transaction receipt.
/// This structure is indexed (i.e. uses [`IndexMap`]s where [`Vec`]s could be used) for convenience
/// and performance, since both the source (i.e. Track) and the sink (i.e. Database and API) operate
/// on indexed structures too.
/// This structure maintains partial information on the order of operations (please see individual
/// fields for details), since the end users care about it. Please note that this means multiple
/// instances of [`StateUpdates`] can represent the same transform of state store (i.e. differing
/// only by order of some operations), and hence it is not 100% "canonical form".
#[derive(Debug, Clone, PartialEq, Eq, Sbor, Default)]
pub struct StateUpdates {
    /// Indexed Node-level updates, captured in the order of first update operation to a Node.
    pub by_node: IndexMap<NodeId, NodeStateUpdates>,
}

impl StateUpdates {
    /// Starts a Node-level update.
    pub fn of_node(&mut self, node_id: NodeId) -> &mut NodeStateUpdates {
        self.by_node
            .entry(node_id)
            .or_insert_with(|| NodeStateUpdates::Delta {
                by_partition: index_map_new(),
            })
    }

    pub fn rebuild_without_empty_entries(&self) -> Self {
        Self {
            by_node: self
                .by_node
                .iter()
                .filter_map(|(node_id, by_partition)| {
                    let by_partition = by_partition.without_empty_entries()?;
                    Some((*node_id, by_partition))
                })
                .collect(),
        }
    }

    pub fn into_legacy(self) -> LegacyStateUpdates {
        self.into()
    }
}

/// A description of all updates that happened to a state of a single Node.
/// Note: currently, we do not support any Node-wide changes (e.g. deleting entire Node); however,
/// we use an enum for potential future development.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum NodeStateUpdates {
    /// A "delta" update to a Node, touching only selected Partitions.
    /// Contains indexed Partition-level updates, captured in the order of first update operation to
    /// a Partition.
    Delta {
        by_partition: IndexMap<PartitionNumber, PartitionStateUpdates>,
    },
}

impl Default for NodeStateUpdates {
    fn default() -> Self {
        NodeStateUpdates::Delta {
            by_partition: index_map_new(),
        }
    }
}

impl NodeStateUpdates {
    /// Starts a Partition-level update.
    pub fn of_partition(&mut self, partition_num: PartitionNumber) -> &mut PartitionStateUpdates {
        match self {
            NodeStateUpdates::Delta { by_partition } => {
                by_partition.entry(partition_num).or_default()
            }
        }
    }

    pub fn without_empty_entries(&self) -> Option<Self> {
        match self {
            NodeStateUpdates::Delta { by_partition } => {
                let replaced = by_partition
                    .iter()
                    .filter_map(|(partition_num, partition_state_updates)| {
                        let new_substate = partition_state_updates.without_empty_entries()?;
                        Some((*partition_num, new_substate))
                    })
                    .collect::<IndexMap<_, _>>();
                if replaced.len() > 0 {
                    Some(NodeStateUpdates::Delta {
                        by_partition: replaced,
                    })
                } else {
                    None
                }
            }
        }
    }
}

/// A description of all updates that happened to a state of a single Partition.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum PartitionStateUpdates {
    /// A "delta" update to a Partition, touching only selected Substates.
    /// Contains indexed Substate-level updates, captured in the order of first update operation to
    /// a Substate.
    Delta {
        by_substate: IndexMap<SubstateKey, DatabaseUpdate>,
    },
    /// A batch update.
    Batch(BatchPartitionStateUpdate),
}

impl Default for PartitionStateUpdates {
    fn default() -> Self {
        PartitionStateUpdates::Delta {
            by_substate: index_map_new(),
        }
    }
}

impl PartitionStateUpdates {
    /// Resets the partition to an empty state.
    pub fn delete(&mut self) {
        *self = PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset {
            new_substate_values: index_map_new(),
        });
    }

    /// Applies the given updates on top of the current updates to the partition.
    pub fn update_substates(
        &mut self,
        updates: impl IntoIterator<Item = (SubstateKey, DatabaseUpdate)>,
    ) {
        match self {
            PartitionStateUpdates::Delta { by_substate } => by_substate.extend(updates),
            PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset {
                new_substate_values,
            }) => {
                for (substate_key, database_update) in updates {
                    match database_update {
                        DatabaseUpdate::Set(new_value) => {
                            new_substate_values.insert(substate_key, new_value);
                        }
                        DatabaseUpdate::Delete => {
                            let existed = new_substate_values.swap_remove(&substate_key).is_some();
                            if !existed {
                                panic!("inconsistent update: delete of substate {:?} not existing in reset partition", substate_key);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn without_empty_entries(&self) -> Option<Self> {
        match self {
            PartitionStateUpdates::Delta { by_substate } => {
                if by_substate.len() > 0 {
                    Some(PartitionStateUpdates::Delta {
                        by_substate: by_substate.clone(),
                    })
                } else {
                    None
                }
            }
            PartitionStateUpdates::Batch(x) => {
                // We shouldn't filter out batch updates like resets, even if they set nothing new
                Some(PartitionStateUpdates::Batch(x.clone()))
            }
        }
    }
}

/// A description of a batch update affecting an entire Partition.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum BatchPartitionStateUpdate {
    /// A reset, dropping all Substates of a partition and replacing them with a new set.
    /// Contains indexed new Substate values, captured in the order of creation of a Substate.
    Reset {
        new_substate_values: IndexMap<SubstateKey, DbSubstateValue>,
    },
}

impl StateUpdates {
    /// Uses the given [`DatabaseKeyMapper`] to express self using database-level key encoding.
    pub fn create_database_updates<M: DatabaseKeyMapper>(&self) -> DatabaseUpdates {
        DatabaseUpdates {
            node_updates: self
                .by_node
                .iter()
                .map(|(node_id, node_state_updates)| {
                    (
                        M::to_db_node_key(node_id),
                        node_state_updates.create_database_updates::<M>(),
                    )
                })
                .collect(),
        }
    }
}

impl NodeStateUpdates {
    /// Uses the given [`DatabaseKeyMapper`] to express self using database-level key encoding.
    pub fn create_database_updates<M: DatabaseKeyMapper>(&self) -> NodeDatabaseUpdates {
        match self {
            NodeStateUpdates::Delta { by_partition } => NodeDatabaseUpdates {
                partition_updates: by_partition
                    .iter()
                    .map(|(partition_num, partition_state_updates)| {
                        (
                            M::to_db_partition_num(*partition_num),
                            partition_state_updates.create_database_updates::<M>(),
                        )
                    })
                    .collect(),
            },
        }
    }
}

impl PartitionStateUpdates {
    /// Uses the given [`DatabaseKeyMapper`] to express self using database-level key encoding.
    pub fn create_database_updates<M: DatabaseKeyMapper>(&self) -> PartitionDatabaseUpdates {
        match self {
            PartitionStateUpdates::Delta { by_substate } => PartitionDatabaseUpdates::Delta {
                substate_updates: by_substate
                    .iter()
                    .map(|(key, update)| (M::to_db_sort_key(key), update.clone()))
                    .collect(),
            },
            PartitionStateUpdates::Batch(batch) => batch.create_database_updates::<M>(),
        }
    }
}

impl BatchPartitionStateUpdate {
    /// Uses the given [`DatabaseKeyMapper`] to express self using database-level key encoding.
    pub fn create_database_updates<M: DatabaseKeyMapper>(&self) -> PartitionDatabaseUpdates {
        match self {
            BatchPartitionStateUpdate::Reset {
                new_substate_values,
            } => PartitionDatabaseUpdates::Reset {
                new_substate_values: new_substate_values
                    .iter()
                    .map(|(key, value)| (M::to_db_sort_key(key), value.clone()))
                    .collect(),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeSubstate {
    pub value: IndexedOwnedScryptoValue,
}

impl RuntimeSubstate {
    pub fn new(value: IndexedOwnedScryptoValue) -> Self {
        Self { value }
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
    pub fn into_value(self) -> Option<IndexedOwnedScryptoValue> {
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
    ReadExistAndWrite(IndexedOwnedScryptoValue, Write),
    ReadNonExistAndWrite(RuntimeSubstate),
    WriteOnly(Write),
    Garbage,
}

impl TrackedSubstate {
    pub fn size(&self) -> usize {
        // `substate_key` is accounted as part of the CanonicalSubstateKey
        self.substate_value.size()
    }
}

impl TrackedSubstateValue {
    pub fn size(&self) -> usize {
        match self {
            TrackedSubstateValue::New(x) => x.value.payload_len(),
            TrackedSubstateValue::ReadOnly(r) => match r {
                ReadOnly::NonExistent => 0,
                ReadOnly::Existent(x) => x.value.payload_len(),
            },
            TrackedSubstateValue::ReadExistAndWrite(e, w) => {
                e.payload_len()
                    + match w {
                        Write::Update(x) => x.value.payload_len(),
                        Write::Delete => 0,
                    }
            }
            TrackedSubstateValue::ReadNonExistAndWrite(x) => x.value.payload_len(),
            TrackedSubstateValue::WriteOnly(w) => match w {
                Write::Update(x) => x.value.payload_len(),
                Write::Delete => 0,
            },
            TrackedSubstateValue::Garbage => 0,
        }
    }

    pub fn get_runtime_substate_mut(&mut self) -> Option<&mut RuntimeSubstate> {
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

    pub fn get(&self) -> Option<&IndexedOwnedScryptoValue> {
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

    pub fn set(&mut self, value: IndexedOwnedScryptoValue) {
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
                *self = new_tracked;
            }
            TrackedSubstateValue::ReadOnly(ReadOnly::Existent(old)) => {
                let new_tracked = TrackedSubstateValue::ReadExistAndWrite(
                    old.value.clone(),
                    Write::Update(RuntimeSubstate::new(value)),
                );
                *self = new_tracked;
            }
            TrackedSubstateValue::ReadExistAndWrite(_, write @ Write::Delete)
            | TrackedSubstateValue::WriteOnly(write @ Write::Delete) => {
                *write = Write::Update(RuntimeSubstate::new(value));
            }
        };
    }

    pub fn take(&mut self) -> Option<IndexedOwnedScryptoValue> {
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

    pub fn into_value(self) -> Option<IndexedOwnedScryptoValue> {
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

pub fn to_state_updates<M: DatabaseKeyMapper + 'static>(
    tracked: TrackedSubstates,
) -> (IndexSet<NodeId>, StateUpdates) {
    let mut new_nodes = index_set_new();
    let mut system_updates = index_map_new();
    for (node_id, tracked_node) in tracked.tracked_nodes {
        if tracked_node.is_new {
            new_nodes.insert(node_id);
        }

        for (partition_num, tracked_partition) in tracked_node.tracked_partitions {
            let mut partition_updates = index_map_new();
            for tracked in tracked_partition.substates.into_values() {
                let update = match tracked.substate_value {
                    TrackedSubstateValue::ReadOnly(..) | TrackedSubstateValue::Garbage => None,
                    TrackedSubstateValue::ReadNonExistAndWrite(substate)
                    | TrackedSubstateValue::New(substate) => {
                        Some(DatabaseUpdate::Set(substate.value.into_payload_bytes()))
                    }
                    TrackedSubstateValue::ReadExistAndWrite(_, write)
                    | TrackedSubstateValue::WriteOnly(write) => match write {
                        Write::Delete => Some(DatabaseUpdate::Delete),
                        Write::Update(substate) => {
                            Some(DatabaseUpdate::Set(substate.value.into_payload_bytes()))
                        }
                    },
                };
                if let Some(update) = update {
                    partition_updates.insert(tracked.substate_key, update);
                }
            }
            system_updates.insert((node_id.clone(), partition_num), partition_updates);
        }
    }

    (
        new_nodes,
        StateUpdates::from(LegacyStateUpdates {
            partition_deletions: tracked.deleted_partitions,
            system_updates,
        }),
    )
}

pub struct IterationCountedIter<'a, E> {
    pub iter: Box<
        dyn Iterator<Item = Result<(DbSortKey, (SubstateKey, IndexedOwnedScryptoValue)), E>> + 'a,
    >,
    pub num_iterations: u32,
}

impl<'a, E> IterationCountedIter<'a, E> {
    pub fn new(
        iter: Box<
            dyn Iterator<Item = Result<(DbSortKey, (SubstateKey, IndexedOwnedScryptoValue)), E>>
                + 'a,
        >,
    ) -> Self {
        Self {
            iter,
            num_iterations: 0u32,
        }
    }
}

impl<'a, E> Iterator for IterationCountedIter<'a, E> {
    type Item = Result<(DbSortKey, (SubstateKey, IndexedOwnedScryptoValue)), E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.num_iterations = self.num_iterations + 1;
        self.iter.next()
    }
}
