use crate::hash_tree::types::{LeafKey, LeafNode, SPARSE_MERKLE_PLACEHOLDER_HASH};
use jellyfish::JellyfishMerkleTree;
use radix_engine_common::crypto::Hash;
use radix_engine_store_interface::interface::{
    DbNodeKey, DbPartitionKey, DbPartitionNum, DbSortKey, DbSubstateKey,
};
use tree_store::{ReadableTreeStore, TreeNode, TreeStore, WriteableTreeStore};
use types::{NibblePath, NodeKey, Version};
use utils::copy_u8_array;
use utils::prelude::vec;
use utils::rust::collections::{index_map_new, IndexMap};
use utils::rust::vec::Vec;

pub mod hash_tree_facade;
pub mod tree_store;

// The sources copied from Aptos (the `jellyfish` and `types` modules) contain support for
// generating proofs, which we plan to use in near future. Hence, we do not delete that code, but
// suppress warnings.

#[allow(dead_code)]
mod jellyfish;
#[cfg(test)]
mod test;
#[allow(dead_code)]
mod types;

/// A change of a hash of value associated with some ID.
/// External API only uses it for hashes of substates (see `SubstateHashChange`), but internally we
/// also use it for lower-level hashes of JMT nodes of different tiers.
#[derive(Debug)]
pub struct IdHashChange<I> {
    /// ID.
    id: I,
    /// A hash after the change, or `None` if this change denotes a delete.
    hash_change: Option<Hash>,
}

impl<I> IdHashChange<I> {
    pub fn new(id: I, hash_change: Option<Hash>) -> Self {
        Self { id, hash_change }
    }
}

/// A top-level `IdHashChange`, representing an actual change of a specific substate's hash.
pub type SubstateHashChange = IdHashChange<DbSubstateKey>;

/// Inserts a new set of nodes at version `node_root_version` + 1 into the "3-Tier JMT" persisted
/// within the given `TreeStore`.
/// In a traditional JMT, this inserts a new leaf node for each given "change", together with an
/// entire new "parent chain" leading from that leaf to a new root node (common for all of them).
/// In our instantiation of the JMT, we first update all touched Substate-Tier JMTs, then we update
/// all touched Partition-Tier JMTs and then we update the single ReNode-Tier tree.
/// All nodes that became stale precisely due to this (i.e. not any previous) operation will be
/// reported before the function returns (see `WriteableTreeStore::record_stale_node`).
/// Returns the hash of the newly-created root (i.e. representing state at version
/// `node_root_version` + 1).
///
/// # Panics
/// Panics if a root node for `node_root_version` does not exist. The caller should use `None` to
/// denote an empty, initial state of the tree (i.e. inserting at version 1).
pub fn put_at_next_version<S: TreeStore>(
    node_tier_store: &mut S,
    node_root_version: Option<Version>,
    changes: Vec<SubstateHashChange>,
) -> Hash {
    let next_version = node_root_version.unwrap_or(0) + 1;
    let node_changes = index_by_node_key_and_partition_num(changes)
        .into_iter()
        .map(|(node_key, substate_changes_by_partition)| {
            let partition_root_version =
                get_lower_tier_root_version(node_tier_store, node_root_version, &node_key);
            let mut partition_tier_store = NestedTreeStore::new(node_tier_store, node_key.clone());
            let partition_changes = substate_changes_by_partition
                .into_iter()
                .map(|(partition_num, substate_changes)| {
                    let partition_key = vec![partition_num];
                    let substate_root_version = get_lower_tier_root_version(
                        &partition_tier_store,
                        partition_root_version,
                        &partition_key,
                    );
                    let mut substate_tier_store =
                        NestedTreeStore::new(&mut partition_tier_store, partition_key);
                    let partition_hash = put_id_hash_changes(
                        &mut substate_tier_store,
                        substate_root_version,
                        next_version,
                        substate_changes
                            .into_iter()
                            .map(|change| IdHashChange::new(change.id.0, change.hash_change))
                            .collect(),
                    );
                    IdHashChange::new(partition_num, partition_hash)
                })
                .collect::<Vec<_>>();
            let node_hash_change = put_id_hash_changes(
                &mut partition_tier_store,
                partition_root_version,
                next_version,
                partition_changes
                    .into_iter()
                    .map(|change| IdHashChange::new(vec![change.id], change.hash_change))
                    .collect(),
            );
            IdHashChange::new(node_key, node_hash_change)
        })
        .collect::<Vec<_>>();
    put_id_hash_changes(
        node_tier_store,
        node_root_version,
        next_version,
        node_changes,
    )
    .unwrap_or(SPARSE_MERKLE_PLACEHOLDER_HASH)
}

pub fn list_substate_hashes_at_version<S: ReadableTreeStore>(
    node_tier_store: &mut S,
    node_root_version: Version,
) -> IndexMap<DbPartitionKey, IndexMap<DbSortKey, Hash>> {
    let mut by_db_partition = index_map_new();
    for node_tier_leaf in list_leaves(node_tier_store, node_root_version) {
        let db_node_key = node_tier_leaf.leaf_key().bytes.clone();
        let mut partition_tier_store = NestedTreeStore::new(node_tier_store, db_node_key.clone());
        for partition_tier_leaf in
            list_leaves(&mut partition_tier_store, node_tier_leaf.payload().clone())
        {
            let db_partition_num =
                DbPartitionNum::from_be_bytes(copy_u8_array(&partition_tier_leaf.leaf_key().bytes));
            let mut substate_tier_store =
                NestedTreeStore::new(&mut partition_tier_store, vec![db_partition_num]);
            let mut by_db_sort_key = index_map_new();
            for substate_tier_leaf in list_leaves(
                &mut substate_tier_store,
                partition_tier_leaf.payload().clone(),
            ) {
                by_db_sort_key.insert(
                    DbSortKey(substate_tier_leaf.leaf_key().bytes.clone()),
                    substate_tier_leaf.value_hash(),
                );
            }
            by_db_partition.insert(
                DbPartitionKey {
                    node_key: db_node_key.clone(),
                    partition_num: db_partition_num,
                },
                by_db_sort_key,
            );
        }
    }
    by_db_partition
}

// only internals below

fn list_leaves<S: ReadableTreeStore>(
    tree_store: &mut S,
    version: Version,
) -> Vec<LeafNode<Version>> {
    let mut leaves = Vec::new();
    list_leaves_recursively(tree_store, NodeKey::new_empty_path(version), &mut leaves);
    leaves
}

fn list_leaves_recursively<S: ReadableTreeStore>(
    tree_store: &mut S,
    key: NodeKey,
    results: &mut Vec<LeafNode<Version>>,
) {
    let Some(node) = tree_store.get_node(&key) else {
        panic!("{:?} referenced but not found in the storage", key);
    };
    match node {
        TreeNode::Internal(internal) => {
            for child in internal.children {
                list_leaves_recursively(
                    tree_store,
                    key.gen_child_node_key(child.version, child.nibble),
                    results,
                );
            }
        }
        TreeNode::Leaf(leaf) => {
            results.push(LeafNode::from(&key, &leaf));
        }
        TreeNode::Null => {}
    };
}

fn get_lower_tier_root_version<S: ReadableTreeStore>(
    store: &S,
    version: Option<Version>,
    leaf_bytes: &[u8],
) -> Option<Version> {
    version.and_then(|version| {
        let leaf_node_data = JellyfishMerkleTree::new(store)
            .get_with_proof(&LeafKey::new(leaf_bytes), version)
            .unwrap()
            .0;
        leaf_node_data.map(|(_hash, last_hash_change_version, _version)| last_hash_change_version)
    })
}

fn index_by_node_key_and_partition_num(
    changes: Vec<SubstateHashChange>,
) -> IndexMap<DbNodeKey, IndexMap<DbPartitionNum, Vec<IdHashChange<DbSortKey>>>> {
    let mut by_db_node_key = index_map_new();
    for IdHashChange { id, hash_change } in changes {
        let (
            DbPartitionKey {
                node_key,
                partition_num,
            },
            sort_key,
        ) = id;
        by_db_node_key
            .entry(node_key)
            .or_insert_with(|| index_map_new())
            .entry(partition_num)
            .or_insert_with(|| Vec::new())
            .push(IdHashChange::new(sort_key, hash_change));
    }
    by_db_node_key
}

fn put_id_hash_changes<S: TreeStore>(
    store: &mut S,
    current_version: Option<Version>,
    next_version: Version,
    changes: Vec<IdHashChange<Vec<u8>>>,
) -> Option<Hash> {
    let root_hash = put_leaf_changes(
        store,
        current_version,
        next_version,
        changes
            .into_iter()
            .map(|change| to_leaf_change(change, next_version))
            .collect(),
    );
    if root_hash == SPARSE_MERKLE_PLACEHOLDER_HASH {
        None
    } else {
        Some(root_hash)
    }
}

fn to_leaf_change(change: IdHashChange<Vec<u8>>, version: Version) -> LeafChange {
    LeafChange {
        key: LeafKey { bytes: change.id },
        new_payload: change.hash_change.map(|value_hash| (value_hash, version)),
    }
}

struct LeafChange {
    key: LeafKey,
    new_payload: Option<(Hash, Version)>,
}

fn put_leaf_changes<S: TreeStore>(
    store: &mut S,
    current_version: Option<Version>,
    next_version: Version,
    changes: Vec<LeafChange>,
) -> Hash {
    let (root_hash, update_result) = JellyfishMerkleTree::new(store)
        .batch_put_value_set(
            changes
                .iter()
                .map(|change| (&change.key, change.new_payload.as_ref()))
                .collect(),
            None,
            current_version,
            next_version,
        )
        .expect("error while reading tree during put");
    for (key, node) in update_result.node_batch.into_iter().flatten() {
        let node = TreeNode::from(&key, &node);
        store.insert_node(key, node)
    }
    for key in update_result.stale_node_index_batch.into_iter().flatten() {
        store.record_stale_node(key.node_key);
    }
    root_hash
}

struct NestedTreeStore<'s, S> {
    underlying: &'s mut S,
    key_prefix_bytes: Vec<u8>,
}

impl<'s, S> NestedTreeStore<'s, S> {
    const TIER_SEPARATOR: u8 = b'_';

    pub fn new(underlying: &'s mut S, parent_tier_key_bytes: Vec<u8>) -> NestedTreeStore<'s, S> {
        let mut key_prefix_bytes = parent_tier_key_bytes;
        key_prefix_bytes.push(Self::TIER_SEPARATOR);
        NestedTreeStore {
            underlying,
            key_prefix_bytes,
        }
    }

    fn prefixed(&self, key: &NodeKey) -> NodeKey {
        NodeKey::new(
            key.version(),
            NibblePath::from_iter(
                NibblePath::new_even(self.key_prefix_bytes.clone())
                    .nibbles()
                    .chain(key.nibble_path().nibbles()),
            ),
        )
    }
}

impl<'s, S: ReadableTreeStore> ReadableTreeStore for NestedTreeStore<'s, S> {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.underlying.get_node(&self.prefixed(key))
    }
}

impl<'s, S: WriteableTreeStore> WriteableTreeStore for NestedTreeStore<'s, S> {
    fn insert_node(&mut self, key: NodeKey, node: TreeNode) {
        self.underlying.insert_node(self.prefixed(&key), node);
    }

    fn record_stale_node(&mut self, key: NodeKey) {
        self.underlying.record_stale_node(self.prefixed(&key));
    }
}
