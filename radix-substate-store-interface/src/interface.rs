use radix_common::Sbor;
use radix_rust::prelude::index_map_new;
use radix_rust::rust::boxed::Box;
use radix_rust::rust::collections::IndexMap;
use radix_rust::rust::vec::Vec;

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

    /// A reset, dropping all Substates of a partition and replacing them with a new set.
    Reset {
        new_substate_values: IndexMap<DbSortKey, DbSubstateValue>,
    },
}

impl PartitionDatabaseUpdates {
    /// Returns an effective new Substate value *upserted* under the given `sort_key` (i.e. after
    /// hypothetically applying this Partition update).
    /// Please note that this method only cares about upserts - i.e. returns [`None`] either if the
    /// substate was unaffected, or if it was deleted by this update.
    ///
    /// This method is useful for index-updating logic which does not care about the nature of the
    /// Partition update (i.e. delta vs reset).
    pub fn get_upserted_value(&self, sort_key: &DbSortKey) -> Option<&DbSubstateValue> {
        match self {
            Self::Delta { substate_updates } => {
                substate_updates
                    .get(sort_key)
                    .and_then(|update| match update {
                        DatabaseUpdate::Set(value) => Some(value),
                        DatabaseUpdate::Delete => None,
                    })
            }
            Self::Reset {
                new_substate_values,
            } => new_substate_values.get(sort_key),
        }
    }
}

impl Default for PartitionDatabaseUpdates {
    fn default() -> Self {
        Self::Delta {
            substate_updates: index_map_new(),
        }
    }
}

/// An update of a single substate's value.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Sbor, PartialOrd, Ord)]
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

    /// Iterates over all entries of the given partition (starting either from the beginning, or
    /// from the given [`DbSortKey`]), in a lexicographical order (ascending) of the [`DbSortKey`]s.
    /// Note: If the exact given starting key does not exist, the iteration starts with its
    /// immediate successor.
    fn list_entries_from(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_>;

    /// Iterates over all entries of the given partition, in a lexicographical order (ascending)
    /// of the [`DbSortKey`]s.
    /// This is a convenience method, equivalent to [`Self::list_entries_from()`] with the starting
    /// key set to [`None`].
    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        self.list_entries_from(partition_key, None)
    }
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
