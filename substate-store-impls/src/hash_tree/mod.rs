pub mod hash_tree_facade;
pub mod tree_store;

pub mod entity_tier;
pub mod partition_tier;
pub mod substate_tier;
pub mod tier_framework;

use entity_tier::EntityTier;
use radix_engine_common::crypto::Hash;
use substate_store_interface::interface::*;
use tree_store::*;
use types::*;
use utils::prelude::*;

use self::{
    partition_tier::PartitionTier, substate_tier::SubstateTier, tier_framework::IterableLeaves,
};

// The sources copied from Aptos (the `jellyfish` and `types` modules) contain support for
// generating proofs, which we plan to use in near future. Hence, we do not delete that code, but
// suppress warnings.

#[allow(dead_code)]
mod jellyfish;
#[cfg(test)]
mod test;
#[allow(dead_code)]
mod types;

/// Inserts a new set of nodes at version `current_state_version` + 1 into the "3-Tier JMT" persisted
/// within the given `TreeStore`.
/// In a traditional JMT, this inserts a new leaf node for each given "change", together with an
/// entire new "parent chain" leading from that leaf to a new root node (common for all of them).
/// In our instantiation of the JMT, we first update all touched Substate-Tier JMTs, then we update
/// all touched Partition-Tier JMTs and then we update the single ReNode-Tier tree.
/// All nodes that became stale precisely due to this (i.e. not any previous) operation will be
/// reported before the function returns (see `WriteableTreeStore::record_stale_node`).
/// Returns the hash of the newly-created root (i.e. representing state at version
/// `current_state_version` + 1).
///
/// # Panics
/// Panics if a root node for `current_state_version` does not exist. The caller should use `None` to
/// denote an empty, initial state of the tree (i.e. inserting at version 1).
pub fn put_at_next_version<S: TreeStore>(
    tree_store: &S,
    current_state_version: Option<Version>,
    database_updates: &DatabaseUpdates,
) -> Hash {
    let entity_tier = EntityTier::new(tree_store, current_state_version);
    let next_state_version = current_state_version.unwrap_or(0) + 1;
    entity_tier
        .put_all_entity_updates(next_state_version, database_updates)
        .unwrap_or(SPARSE_MERKLE_PLACEHOLDER_HASH)
}

pub fn list_substate_hashes_at_version<S: ReadableTreeStore>(
    tree_store: &S,
    root_state_version: Version,
) -> IndexMap<DbPartitionKey, IndexMap<DbSortKey, Hash>> {
    let entity_tier: EntityTier<'_, S> = EntityTier::new(tree_store, Some(root_state_version));
    let mut by_db_partition = index_map_new();
    for (_, entity_key, entity_version) in entity_tier.iter_leaves() {
        let partition_tier =
            PartitionTier::new(tree_store, Some(entity_version), entity_key.clone());
        for (_, partition, partition_version) in partition_tier.iter_leaves() {
            let substate_tier = SubstateTier::new(
                tree_store,
                Some(partition_version),
                entity_key.clone(),
                partition,
            );
            let mut by_db_sort_key = index_map_new();
            for (value_hash, sort_key, _) in substate_tier.iter_leaves() {
                by_db_sort_key.insert(sort_key, value_hash);
            }
            let partition_key = DbPartitionKey {
                node_key: entity_key.clone(),
                partition_num: partition,
            };
            by_db_partition.insert(partition_key, by_db_sort_key);
        }
    }
    by_db_partition
}
