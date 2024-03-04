use crate::hash_tree::tree_store::StaleTreePart;
use crate::hash_tree::types::{LeafKey, LeafNode, Node, SPARSE_MERKLE_PLACEHOLDER_HASH};
use crate::hash_tree::Payload::SubstateValue;
use jellyfish::JellyfishMerkleTree;
use radix_engine_common::crypto::{hash, Hash};
use substate_store_interface::interface::{
    DatabaseUpdate, DatabaseUpdates, DbNodeKey, DbPartitionKey, DbPartitionNum, DbSortKey,
    DbSubstateValue, NodeDatabaseUpdates, PartitionDatabaseUpdates,
};
use tree_store::{ReadableTreeStore, TreeNode, TreeStore, WriteableTreeStore};
use types::{NibblePath, NodeKey, Version};
use utils::copy_u8_array;
use utils::prelude::vec;
use utils::rust::collections::{index_map_new, IndexMap};
use utils::rust::ops::Deref;
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
    node_tier_store: &S,
    node_root_version: Option<Version>,
    database_updates: &DatabaseUpdates,
) -> Hash {
    let next_version = node_root_version.unwrap_or(0) + 1;
    let node_hash_changes = database_updates
        .node_updates
        .iter()
        .map(|(node_key, node_database_updates)| {
            LeafChange::for_node(
                node_key,
                apply_node_database_updates(
                    node_tier_store,
                    node_root_version,
                    node_key,
                    node_database_updates,
                    next_version,
                ),
            )
        })
        .collect::<Vec<_>>();
    put_leaf_hash_changes(
        node_tier_store,
        node_root_version,
        next_version,
        node_hash_changes,
    )
    .unwrap_or(SPARSE_MERKLE_PLACEHOLDER_HASH)
}

pub fn list_substate_hashes_at_version<S: ReadableTreeStore>(
    node_tier_store: &S,
    node_root_version: Version,
) -> IndexMap<DbPartitionKey, IndexMap<DbSortKey, Hash>> {
    let mut by_db_partition = index_map_new();
    for node_tier_leaf in list_leaves(node_tier_store, node_root_version) {
        let db_node_key = node_tier_leaf.leaf_key().bytes.clone();
        let partition_tier_store = NestedTreeStore::new(node_tier_store, db_node_key.clone());
        for partition_tier_leaf in
            list_leaves(&partition_tier_store, node_tier_leaf.payload().clone())
        {
            let db_partition_num =
                DbPartitionNum::from_be_bytes(copy_u8_array(&partition_tier_leaf.leaf_key().bytes));
            let substate_tier_store =
                NestedTreeStore::new(&partition_tier_store, vec![db_partition_num]);
            let mut by_db_sort_key = index_map_new();
            for substate_tier_leaf in
                list_leaves(&substate_tier_store, partition_tier_leaf.payload().clone())
            {
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

fn list_leaves<S: ReadableTreeStore>(tree_store: &S, version: Version) -> Vec<LeafNode<Version>> {
    let mut leaves = Vec::new();
    list_leaves_recursively(tree_store, NodeKey::new_empty_path(version), &mut leaves);
    leaves
}

fn list_leaves_recursively<S: ReadableTreeStore>(
    tree_store: &S,
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

fn apply_node_database_updates<S: TreeStore>(
    node_tier_store: &S,
    node_root_version: Option<Version>,
    node_key: &DbNodeKey,
    node_database_updates: &NodeDatabaseUpdates,
    next_version: Version,
) -> Option<Hash> {
    let partition_root_version =
        get_lower_tier_root_version(node_tier_store, node_root_version, node_key);
    let partition_tier_store = NestedTreeStore::new(node_tier_store, node_key.clone());
    let partition_hash_changes = node_database_updates
        .partition_updates
        .iter()
        .map(|(partition_num, partition_database_updates)| {
            LeafChange::for_partition(
                partition_num,
                apply_partition_database_updates(
                    &partition_tier_store,
                    partition_root_version,
                    partition_num,
                    partition_database_updates,
                    next_version,
                ),
            )
        })
        .collect::<Vec<_>>();
    put_leaf_hash_changes(
        &partition_tier_store,
        partition_root_version,
        next_version,
        partition_hash_changes,
    )
}

fn apply_partition_database_updates<S: TreeStore>(
    partition_tier_store: &S,
    partition_root_version: Option<Version>,
    partition_num: &DbPartitionNum,
    partition_database_updates: &PartitionDatabaseUpdates,
    next_version: Version,
) -> Option<Hash> {
    let partition_key = vec![*partition_num];
    let substate_root_version =
        get_lower_tier_root_version(partition_tier_store, partition_root_version, &partition_key);
    let substate_tier_store = NestedTreeStore::new(partition_tier_store, partition_key);
    match partition_database_updates {
        PartitionDatabaseUpdates::Delta { substate_updates } => put_leaf_hash_changes(
            &substate_tier_store,
            substate_root_version,
            next_version,
            substate_updates
                .into_iter()
                .map(|(sort_key, update)| LeafChange::for_substate_upsert(sort_key, update))
                .collect(),
        ),
        PartitionDatabaseUpdates::Reset {
            new_substate_values,
        } => {
            if let Some(substate_root_version) = substate_root_version {
                substate_tier_store.record_stale_tree_part(StaleTreePart::Subtree(
                    NodeKey::new_empty_path(substate_root_version),
                ));
            }
            put_leaf_hash_changes(
                &substate_tier_store,
                None,
                next_version,
                new_substate_values
                    .into_iter()
                    .map(|(sort_key, value)| LeafChange::for_substate_reset(sort_key, value))
                    .collect(),
            )
        }
    }
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

enum Payload<'a> {
    NestedTreeRootHash(Hash),
    SubstateValue(&'a DbSubstateValue),
}

impl Payload<'_> {
    pub fn value_hash(&self) -> Hash {
        match self {
            Self::NestedTreeRootHash(hash) => hash.clone(),
            Self::SubstateValue(value) => hash(*value),
        }
    }
}

struct LeafChange<'a> {
    key_bytes: Vec<u8>,
    new_payload: Option<Payload<'a>>,
}

impl<'a> LeafChange<'a> {
    pub fn for_node(node_key: &DbNodeKey, new_root_hash: Option<Hash>) -> Self {
        Self {
            key_bytes: node_key.clone(),
            new_payload: new_root_hash.map(|hash| Payload::NestedTreeRootHash(hash)),
        }
    }

    pub fn for_partition(partition_num: &DbPartitionNum, new_root_hash: Option<Hash>) -> Self {
        Self {
            key_bytes: vec![*partition_num],
            new_payload: new_root_hash.map(|hash| Payload::NestedTreeRootHash(hash)),
        }
    }

    pub fn for_substate_upsert(sort_key: &DbSortKey, update: &'a DatabaseUpdate) -> Self {
        Self {
            key_bytes: sort_key.0.clone(),
            new_payload: match update {
                DatabaseUpdate::Set(value) => Some(SubstateValue(value)),
                DatabaseUpdate::Delete => None,
            },
        }
    }

    pub fn for_substate_reset(sort_key: &DbSortKey, value: &'a DbSubstateValue) -> Self {
        Self {
            key_bytes: sort_key.0.clone(),
            new_payload: Some(SubstateValue(value)),
        }
    }
}

fn put_leaf_hash_changes<S: TreeStore>(
    store: &S,
    current_version: Option<Version>,
    next_version: Version,
    changes: Vec<LeafChange>,
) -> Option<Hash> {
    // We need to re-allocate it like this in order to:
    // - construct the right shape (i.e. a reference to a tuple) of items passed to JMT,
    // - and recover Substate values based on `LeafKey` from the update batch returned by JMT.
    let changes = changes
        .into_iter()
        .map(|change| {
            (
                LeafKey {
                    bytes: change.key_bytes,
                },
                change
                    .new_payload
                    .map(|payload| ((payload.value_hash(), next_version), payload)),
            )
        })
        .collect::<IndexMap<_, _>>();
    let (root_hash, update_result) = JellyfishMerkleTree::new(store)
        .batch_put_value_set(
            changes
                .iter()
                .map(|(key, change)| {
                    (
                        key,
                        change
                            .as_ref()
                            .map(|(hash_and_version, _payload)| hash_and_version),
                    )
                })
                .collect(),
            None,
            current_version,
            next_version,
        )
        .expect("error while reading tree during put");

    for (key, node) in update_result.node_batch.into_iter().flatten() {
        // We promised to associate Substate values; but not all newly-created nodes are leaves:
        if let Node::Leaf(leaf_node) = &node {
            // And not every newly-created leaf comes from a value change: (sometimes it is just a tree re-structuring!)
            if let Some(change) = changes.get(leaf_node.leaf_key()) {
                // Now: if a JMT leaf was created due to value change, then it must have been an upsert:
                let (_hash_and_version, payload) = change.as_ref().expect("unexpected delete");
                // Only the Substate Tier's leaves contain Substate values:
                if let Payload::SubstateValue(substate_value) = payload {
                    store.associate_substate_value(&key, *substate_value);
                }
            }
        }
        let node = TreeNode::from(&key, &node);
        store.insert_node(key, node);
    }
    for key in update_result.stale_node_index_batch.into_iter().flatten() {
        store.record_stale_tree_part(StaleTreePart::Node(key.node_key));
    }

    if root_hash == SPARSE_MERKLE_PLACEHOLDER_HASH {
        None
    } else {
        Some(root_hash)
    }
}

pub struct NestedTreeStore<S> {
    underlying: S,
    key_prefix_bytes: Vec<u8>,
}

impl<S> NestedTreeStore<S> {
    const TIER_SEPARATOR: u8 = b'_';

    pub fn new(underlying: S, parent_tier_key_bytes: Vec<u8>) -> NestedTreeStore<S> {
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

impl<'s, S: Deref<Target = impl ReadableTreeStore> + 's> ReadableTreeStore for NestedTreeStore<S> {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.underlying.get_node(&self.prefixed(key))
    }
}

impl<'s, S: Deref<Target = impl WriteableTreeStore> + 's> WriteableTreeStore
    for NestedTreeStore<S>
{
    fn insert_node(&self, key: NodeKey, node: TreeNode) {
        self.underlying.insert_node(self.prefixed(&key), node);
    }

    fn associate_substate_value(&self, key: &NodeKey, substate_value: &DbSubstateValue) {
        self.underlying
            .associate_substate_value(&self.prefixed(&key), substate_value);
    }

    fn record_stale_tree_part(&self, part: StaleTreePart) {
        self.underlying.record_stale_tree_part(match part {
            StaleTreePart::Node(key) => StaleTreePart::Node(self.prefixed(&key)),
            StaleTreePart::Subtree(key) => StaleTreePart::Subtree(self.prefixed(&key)),
        });
    }
}
