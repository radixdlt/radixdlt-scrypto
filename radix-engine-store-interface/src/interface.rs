use radix_engine_common::Sbor;
use utils::prelude::index_map_new;
use utils::prelude::vec;
use utils::rust::boxed::Box;
use utils::rust::collections::IndexMap;
use utils::rust::vec::Vec;

pub type DbNodeKey = Vec<u8>;

pub type DbPartitionNum = u8;

/// A database-level key of an entire partition.
/// Seen from the higher-level API: it represents a pair (RE Node ID, Module ID).
/// Seen from the lower-level implementation: it is used as a key in the upper-layer tree of our
/// two-layered JMT.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd, Sbor)]
pub struct DbPartitionKey {
    pub node_key: DbNodeKey,
    pub partition_num: DbPartitionNum,
}

impl DbPartitionKey {
    /// Calculates a hypothetical "next partition" key in the database.
    /// This method is suitable for constructing an open right bound of a database key range; the
    /// partition of the returned key may in practice not even exist in the database.
    pub fn next(&self) -> Self {
        self.partition_num
            .checked_add(1)
            .map(|next_partition_num| DbPartitionKey {
                node_key: self.node_key.clone(),
                partition_num: next_partition_num,
            })
            .unwrap_or_else(|| DbPartitionKey {
                node_key: [self.node_key.clone(), vec![0]].concat(),
                partition_num: 0,
            })
    }
}

/// A database-level key of a substate within a known partition.
/// Seen from the higher-level API: it represents a local Substate Key.
/// Seen from the lower-level implementation: it is used as a key in the Substate-Tier JMT.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd, Sbor)]
pub struct DbSortKey(pub Vec<u8>);

/// A fully-specified key of a substate (i.e. specifying its partition and sort key).
pub type DbSubstateKey = (DbPartitionKey, DbSortKey);

/// A raw substate value stored by the database.
pub type DbSubstateValue = Vec<u8>;

/// A key-value entry of a substate within a known partition.
pub type PartitionEntry = (DbSortKey, DbSubstateValue);

/// A canonical description of all database updates to be applied.
/// Note: this struct can be migrated to an enum if we ever have a need for database-wide batch
/// changes (see [`PartitionDatabaseUpdates`] enum).
#[derive(Debug, Clone, PartialEq, Eq, Sbor, Default)]
pub struct DatabaseUpdates {
    /// Node-level updates.
    pub node_updates: IndexMap<DbNodeKey, NodeDatabaseUpdates>,
}

/// A canonical description of specific Node's updates to be applied.
/// Note: this struct can be migrated to an enum if we ever have a need for Node-wide batch changes
/// (see [`PartitionDatabaseUpdates`] enum).
#[derive(Debug, Clone, PartialEq, Eq, Sbor, Default)]
pub struct NodeDatabaseUpdates {
    /// Partition-level updates.
    pub partition_updates: IndexMap<DbPartitionNum, PartitionDatabaseUpdates>,
}

/// A canonical description of specific Partition's updates to be applied.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum PartitionDatabaseUpdates {
    /// A delta change, touching just selected substates.
    Delta {
        substate_updates: IndexMap<DbSortKey, DatabaseUpdate>,
    },
    /// A batch change.
    Batch(BatchPartitionDatabaseUpdate),
}

impl Default for PartitionDatabaseUpdates {
    fn default() -> Self {
        Self::Delta {
            substate_updates: index_map_new(),
        }
    }
}

/// An update affecting entire Partition at once.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum BatchPartitionDatabaseUpdate {
    Reset {
        new_substate_values: IndexMap<DbSortKey, DbSubstateValue>,
    },
}

/// An update of a single substate's value.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Sbor)]
pub enum DatabaseUpdate {
    Set(DbSubstateValue),
    Delete,
}

impl DatabaseUpdates {
    /// Constructs an instance from the given legacy representation (a map of maps), which is only
    /// capable of specifying "deltas" (i.e. individual substate changes; no partition deletes).
    ///
    /// Note: This method is only meant for tests/demos - with regular Engine usage, the
    /// [`DatabaseUpdates`] can be obtained directly from the receipt.
    pub fn from_delta_maps(
        maps: IndexMap<DbPartitionKey, IndexMap<DbSortKey, DatabaseUpdate>>,
    ) -> DatabaseUpdates {
        let mut database_updates = DatabaseUpdates::default();
        for (
            DbPartitionKey {
                node_key,
                partition_num,
            },
            substate_updates,
        ) in maps
        {
            database_updates
                .node_updates
                .entry(node_key)
                .or_default()
                .partition_updates
                .insert(
                    partition_num,
                    PartitionDatabaseUpdates::Delta { substate_updates },
                );
        }
        database_updates
    }
}

/// A read interface between Track and a database vendor.
pub trait SubstateDatabase {
    /// Reads a substate value by its partition and sort key, or [`Option::None`] if missing.
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue>;

    /// Iterates over all entries of the given partition, in a lexicographical order (ascending)
    /// of the [`DbSortKey`]s.
    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_>;
}

/// A write interface between Track and a database vendor.
pub trait CommittableSubstateDatabase {
    /// Commits state changes to the database.
    fn commit(&mut self, database_updates: &DatabaseUpdates);
}

/// A partition listing interface between Track and a database vendor.
pub trait ListableSubstateDatabase {
    /// Iterates over all partition keys, in an arbitrary order.
    fn list_partition_keys(&self) -> Box<dyn Iterator<Item = DbPartitionKey> + '_>;
}
