use radix_engine_derive::ScryptoSbor;
use utils::rust::boxed::Box;
use utils::rust::collections::IndexMap;
use utils::rust::vec::Vec;

/// A database-level key of an entire partition.
/// Seen from the higher-level API: it represents a pair (RE Node ID, Module ID).
/// Seen from the lower-level implementation: it is used as a key in the upper-layer tree of our
/// two-layered JMT.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd, ScryptoSbor)]
pub struct DbPartitionKey(pub Vec<u8>);

/// A database-level key of a substate within a known partition.
/// Seen from the higher-level API: it represents a local Substate Key.
/// Seen from the lower-level implementation: it is used as a key in the lower-layer tree of our
/// two-layered JMT.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd, ScryptoSbor)]
pub struct DbSortKey(pub Vec<u8>);

/// A fully-specified key of a substate (i.e. specifying its partition and sort key).
pub type DbSubstateKey = (DbPartitionKey, DbSortKey);

/// A raw substate value stored by the database.
pub type DbSubstateValue = Vec<u8>;

/// A key-value entry of a substate within a known partition.
pub type PartitionEntry = (DbSortKey, DbSubstateValue);

/// A fully-specified set of substate value updates (aggregated by partition).
pub type DatabaseUpdates = IndexMap<DbPartitionKey, PartitionUpdates>;

/// A set of substate value updates within a known partition.
pub type PartitionUpdates = IndexMap<DbSortKey, DatabaseUpdate>;

/// An update of a single substate values.
#[derive(Debug, Clone, Hash, PartialEq, Eq, ScryptoSbor)]
pub enum DatabaseUpdate {
    Set(DbSubstateValue),
    Delete,
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
