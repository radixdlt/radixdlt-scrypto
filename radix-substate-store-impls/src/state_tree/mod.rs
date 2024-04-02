pub mod state_tree_facade;
pub mod tree_store;

pub mod entity_tier;
pub mod partition_tier;
pub mod substate_tier;
pub mod tier_framework;

use entity_tier::EntityTier;
use radix_common::crypto::Hash;
use radix_rust::prelude::*;
use radix_substate_store_interface::interface::*;
use tree_store::*;
use types::*;

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
    EntityTier::new(tree_store, current_state_version)
        .put_next_version_entity_updates(database_updates)
        .unwrap_or(SPARSE_MERKLE_PLACEHOLDER_HASH)
}

pub fn list_substate_hashes_at_version<S: ReadableTreeStore>(
    tree_store: &S,
    root_state_version: Version,
) -> IndexMap<DbPartitionKey, IndexMap<DbSortKey, Hash>> {
    EntityTier::new(tree_store, Some(root_state_version))
        .iter_entity_partition_tiers_from(None)
        .flat_map(|partition_tier| {
            partition_tier
                .into_iter_partition_substate_tiers_from(None)
                .map(|substate_tier| {
                    let partition_key = substate_tier.partition_key().clone();
                    let by_sort_key = substate_tier
                        .into_iter_substate_summaries_from(None)
                        .map(|substate| (substate.sort_key, substate.value_hash))
                        .collect::<IndexMap<_, _>>();
                    (partition_key, by_sort_key)
                })
        })
        .collect()
}
