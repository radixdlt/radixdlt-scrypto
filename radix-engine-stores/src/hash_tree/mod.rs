use crate::hash_tree::tree_change::{NodeTierChange, PartitionTierChange, TreeChange};
use crate::hash_tree::tree_store::StaleTreePart;
use crate::hash_tree::types::{LeafKey, SPARSE_MERKLE_PLACEHOLDER_HASH};
use jellyfish::JellyfishMerkleTree;
use radix_engine_common::crypto::Hash;
use radix_engine_store_interface::interface::{
    DbNodeKey, DbPartitionKey, DbPartitionNum, DbSubstateKey,
};
use tree_store::{ReadableTreeStore, TreeNode, TreeStore, WriteableTreeStore};
use types::{NibblePath, NodeKey, Version};
use utils::prelude::vec;
use utils::rust::vec::Vec;

pub mod hash_tree_facade;
pub mod tree_store;

mod tree_change;

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

/// A top-level change of hash(es) of a substate (or a group of substates).
pub enum HashChange {
    /// Change to a single substate.
    Single(SubstateHashChange),
    /// Batch change.
    Batch(BatchChange),
}

/// A top-level [`IdHashChange`], representing a change of a single substate's hash.
pub type SubstateHashChange = IdHashChange<DbSubstateKey>;

/// A batch change, representing a (potentially large) number of [`SubstateHashChange`]s in a more
/// compact way (primarily for performance reasons).
pub enum BatchChange {
    /// A deletion of all substates within a specific partition.
    DeletePartition(DbPartitionKey),
}

/// Inserts a new set of nodes at version `node_root_version` + 1 into the "3-Tier JMT" persisted
/// within the given `TreeStore`.
/// The `changes` are applied in order - in particular, a latter change may overwrite some (parts
/// of) any former change.
/// In a traditional JMT, this inserts a new leaf node for each given "change", together with an
/// entire new "parent chain" leading from that leaf to a new root node (common for all of them).
/// In our instantiation of the JMT, we first update all touched Substate-Tier JMTs, then we update
/// all touched Partition-Tier JMTs and then we update the single ReNode-Tier tree.
/// All nodes that became stale precisely due to this (i.e. not any previous) method call will be
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
    changes: Vec<HashChange>,
) -> Hash {
    let next_version = node_root_version.unwrap_or(0) + 1;
    match TreeChange::from(changes) {
        TreeChange::Delta { node_changes } => {
            let node_hash_changes = node_changes
                .into_iter()
                .map(|(node_key, node_tier_change)| {
                    IdHashChange::new(
                        node_key.clone(),
                        apply_node_tier_change(
                            node_tier_store,
                            node_root_version,
                            node_key,
                            node_tier_change,
                            next_version,
                        ),
                    )
                })
                .collect::<Vec<_>>();
            put_id_hash_changes(
                node_tier_store,
                node_root_version,
                next_version,
                node_hash_changes,
            )
            .unwrap_or(SPARSE_MERKLE_PLACEHOLDER_HASH)
        }
    }
}

// only internals below

fn apply_node_tier_change<S: TreeStore>(
    node_tier_store: &mut S,
    node_root_version: Option<Version>,
    node_key: DbNodeKey,
    node_tier_change: NodeTierChange,
    next_version: Version,
) -> Option<Hash> {
    let partition_root_version =
        get_lower_tier_root_version(node_tier_store, node_root_version, &node_key);
    let mut partition_tier_store = NestedTreeStore::new(node_tier_store, node_key.clone());
    match node_tier_change {
        NodeTierChange::Delta { partition_changes } => {
            let partition_hash_changes = partition_changes
                .into_iter()
                .map(|(partition_num, partition_tier_change)| {
                    IdHashChange::new(
                        vec![partition_num],
                        apply_partition_tier_change(
                            &mut partition_tier_store,
                            partition_root_version,
                            partition_num,
                            partition_tier_change,
                            next_version,
                        ),
                    )
                })
                .collect::<Vec<_>>();
            put_id_hash_changes(
                &mut partition_tier_store,
                partition_root_version,
                next_version,
                partition_hash_changes,
            )
        }
    }
}

fn apply_partition_tier_change<S: TreeStore>(
    partition_tier_store: &mut S,
    partition_root_version: Option<Version>,
    partition_num: DbPartitionNum,
    partition_tier_change: PartitionTierChange,
    next_version: Version,
) -> Option<Hash> {
    let partition_key = vec![partition_num];
    let substate_root_version =
        get_lower_tier_root_version(partition_tier_store, partition_root_version, &partition_key);
    let mut substate_tier_store = NestedTreeStore::new(partition_tier_store, partition_key);
    match partition_tier_change {
        PartitionTierChange::Delta { substate_changes } => put_id_hash_changes(
            &mut substate_tier_store,
            substate_root_version,
            next_version,
            substate_changes
                .into_iter()
                .map(|change| IdHashChange::new(change.id.0, change.hash_change))
                .collect(),
        ),
        PartitionTierChange::Reset { substate_hashes } => {
            if let Some(substate_root_version) = substate_root_version {
                substate_tier_store.record_stale_tree_part(StaleTreePart::Subtree(
                    NodeKey::new_empty_path(substate_root_version),
                ));
            }
            put_id_hash_changes(
                &mut substate_tier_store,
                None,
                next_version,
                substate_hashes
                    .into_iter()
                    .map(|(sort_key, hash)| IdHashChange::new(sort_key.0, Some(hash)))
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
        store.record_stale_tree_part(StaleTreePart::Node(key.node_key));
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

    fn record_stale_tree_part(&mut self, part: StaleTreePart) {
        self.underlying.record_stale_tree_part(match part {
            StaleTreePart::Node(key) => StaleTreePart::Node(self.prefixed(&key)),
            StaleTreePart::Subtree(key) => StaleTreePart::Subtree(self.prefixed(&key)),
        });
    }
}
