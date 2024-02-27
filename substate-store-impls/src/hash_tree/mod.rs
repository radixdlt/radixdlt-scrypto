use crate::hash_tree::tree_store::StaleTreePart;
use crate::hash_tree::types::{LeafKey, LeafNode, SPARSE_MERKLE_PLACEHOLDER_HASH};
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

/// Payload stored at each leaf node.
#[derive(Clone, Debug, Ord, PartialOrd, Hash, Eq, PartialEq)]
pub struct ValuePayload {
    /// State version at which the stored value hash was most recently changed.
    ///
    /// In practice, this is a state version at which:
    /// - (in Substate Tier) the represented Substate was most recently upserted,
    /// - or (in upper Tiers) there was any change in the represented lower Tier.
    ///
    /// Please note this can be different than the tree Node's version (i.e. there are cases in which a tree Node is
    /// changed - e.g. gets promoted up after losing all siblings - while its value remains unchanged).
    ///
    /// The version recorded here is used to navigate the 3-Tier JMT structure (i.e. locate the right lower tier root).
    pub last_hash_change_version: Version,

    /// The actual value of the represented Substate.
    ///
    /// May be empty:
    /// - for leaf nodes that do not represent Substates (i.e. leafs of upper Tiers),
    /// - or when the JMT instance is explicitly configured to not store values.
    ///
    /// TODO(potential refactoring): This field is relevant only for Substate Tier, so from object modelling PoV, it
    /// could be a type parameter. A naive refactoring might affect tree storage (which could be problematic for Node),
    /// so special care is needed if we want to pursue this.
    pub value: Option<DbSubstateValue>,
}

impl ValuePayload {
    /// Creates an instance with fields applicable for all Tiers.
    pub fn new(last_hash_change_version: Version) -> Self {
        Self {
            last_hash_change_version,
            value: None,
        }
    }

    /// Augments the instance with the actual Substate value (for Substate Tier, when storing values is enabled).
    pub fn with_value(self, value: DbSubstateValue) -> Self {
        Self {
            last_hash_change_version: self.last_hash_change_version,
            value: Some(value),
        }
    }
}

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
            LeafChange::for_aggregate_update(
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
        for partition_tier_leaf in list_leaves(
            &partition_tier_store,
            node_tier_leaf.payload().last_hash_change_version,
        ) {
            let db_partition_num =
                DbPartitionNum::from_be_bytes(copy_u8_array(&partition_tier_leaf.leaf_key().bytes));
            let substate_tier_store =
                NestedTreeStore::new(&partition_tier_store, vec![db_partition_num]);
            let mut by_db_sort_key = index_map_new();
            for substate_tier_leaf in list_leaves(
                &substate_tier_store,
                partition_tier_leaf.payload().last_hash_change_version,
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
    tree_store: &S,
    version: Version,
) -> Vec<LeafNode<ValuePayload>> {
    let mut leaves = Vec::new();
    list_leaves_recursively(tree_store, NodeKey::new_empty_path(version), &mut leaves);
    leaves
}

fn list_leaves_recursively<S: ReadableTreeStore>(
    tree_store: &S,
    key: NodeKey,
    results: &mut Vec<LeafNode<ValuePayload>>,
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
            LeafChange::for_aggregate_update(
                &vec![*partition_num],
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
                .map(|(sort_key, update)| LeafChange::for_substate_update(sort_key, update))
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
        leaf_node_data.map(|(_hash, payload, _version)| payload.last_hash_change_version)
    })
}

struct LeafChange {
    key_bytes: Vec<u8>,
    hash_change: Option<PayloadChange>,
}

enum PayloadChange {
    SubstateChange(DbSubstateValue),
    AggregateChange(Hash),
}

impl LeafChange {
    pub fn for_substate_update(sort_key: &DbSortKey, update: &DatabaseUpdate) -> Self {
        Self {
            key_bytes: sort_key.0.clone(),
            hash_change: match update {
                DatabaseUpdate::Set(value) => Some(PayloadChange::SubstateChange(value.clone())),
                DatabaseUpdate::Delete => None,
            },
        }
    }

    pub fn for_substate_reset(sort_key: &DbSortKey, value: &DbSubstateValue) -> Self {
        Self {
            key_bytes: sort_key.0.clone(),
            hash_change: Some(PayloadChange::SubstateChange(value.clone())),
        }
    }

    pub fn for_aggregate_update(node_key: &DbNodeKey, lower_tier_root_hash: Option<Hash>) -> Self {
        Self {
            key_bytes: node_key.clone(),
            hash_change: lower_tier_root_hash.map(|hash| PayloadChange::AggregateChange(hash)),
        }
    }

    pub fn to_stored_leaf_change(self, version: Version) -> StoredLeafChange {
        // TODO(wip): pass config here
        let payload = ValuePayload::new(version);
        StoredLeafChange {
            key: LeafKey {
                bytes: self.key_bytes,
            },
            new_payload: self.hash_change.map(|payload_change| match payload_change {
                PayloadChange::SubstateChange(value) => (hash(&value), payload.with_value(value)),
                PayloadChange::AggregateChange(hash) => (hash, payload),
            }),
        }
    }
}

fn put_leaf_hash_changes<S: TreeStore>(
    store: &S,
    current_version: Option<Version>,
    next_version: Version,
    changes: Vec<LeafChange>,
) -> Option<Hash> {
    let root_hash = put_leaf_changes(
        store,
        current_version,
        next_version,
        changes
            .into_iter()
            .map(|change| change.to_stored_leaf_change(next_version))
            .collect(),
    );
    if root_hash == SPARSE_MERKLE_PLACEHOLDER_HASH {
        None
    } else {
        Some(root_hash)
    }
}

/// A lower-level representation of a [`LeafChange`], tailored for obtaining the references as required by the
/// [`JellyfishMerkleTree::batch_put_value_set()`] method.
struct StoredLeafChange {
    key: LeafKey,
    new_payload: Option<(Hash, ValuePayload)>,
}

fn put_leaf_changes<S: TreeStore>(
    store: &S,
    current_version: Option<Version>,
    next_version: Version,
    changes: Vec<StoredLeafChange>,
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

    fn record_stale_tree_part(&self, part: StaleTreePart) {
        self.underlying.record_stale_tree_part(match part {
            StaleTreePart::Node(key) => StaleTreePart::Node(self.prefixed(&key)),
            StaleTreePart::Subtree(key) => StaleTreePart::Subtree(self.prefixed(&key)),
        });
    }
}
