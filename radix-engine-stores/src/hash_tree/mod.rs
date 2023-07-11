use crate::hash_tree::tree_store::{NodePayload, PartitionPayload, SubstatePayload};
use crate::hash_tree::types::{LeafKey, SPARSE_MERKLE_PLACEHOLDER_HASH};
use jellyfish::JellyfishMerkleTree;
use radix_engine_common::crypto::{hash, Hash};
use radix_engine_store_interface::interface::{
    DbNodeKey, DbPartitionKey, DbPartitionNum, DbSortKey, DbSubstateKey,
};
use tree_store::{Payload, ReadableTreeStore, TreeNode, TreeStore, WriteableTreeStore};
use types::{NibblePath, NodeKey, Version};
use utils::rust::collections::{index_map_new, BTreeMap, IndexMap};
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

// TODO(wip): adjust all rustdocs here to reflect the 3-Tier JMT

/// A change of value associated with some ID.
/// External API only uses it for hashes of substates (see `SubstateHashChange`), but internally we
/// split into "change of ReNode layer's leaf" and "change of substate offset's value in the nested
/// tree".
#[derive(Debug)]
pub struct IdChange<I, V> {
    /// ID.
    id: I,
    /// A value after change, or `None` if this change denotes a delete.
    changed: Option<V>,
}

impl<I, V> IdChange<I, V> {
    pub fn new(id: I, changed: Option<V>) -> Self {
        Self { id, changed }
    }
}

/// A top-level `IdChange`, representing an actual change of a specific substate's hashed value.
pub type SubstateHashChange = IdChange<DbSubstateKey, Hash>;

/// Inserts a new set of nodes at version `current_version` + 1 into the "nested JMT" persisted
/// within the given `store`.
/// In a traditional JMT, this inserts a new leaf node for each given "change", together with an
/// entire new "parent chain" leading from that leaf to a new root node (common for all of them).
/// In our instantiation of the JMT, we first update all nested per-`ReNodeModule` trees (i.e. of
/// each {`NodeId`, `ModuleId`} pair encountered in the `changes`), and then we update the
/// single upper-layer tree (representing all `ReNodeModule`).
/// All nodes that became stale precisely due to this (i.e. not any previous) operation will be
/// reported before the function returns (see `WriteableTreeStore::record_stale_node`).
/// Returns the hash of the newly-created root (i.e. representing state at version
/// `current_version` + 1).
///
/// # Panics
/// Panics if a root node for `current_version` does not exist. The caller should use `None` to
/// denote an empty, initial state of the tree (i.e. inserting at version 1).
pub fn put_at_next_version<S: TreeStore<NodePayload> + TreeStore<SubstatePayload>>(
    store: &mut S,
    current_version: Option<Version>,
    changes: Vec<SubstateHashChange>,
) -> Hash {
    let node_changes = index_by_node_key_and_partition_num(changes)
        .into_iter()
        .map(|(node_key, partition_changes)| {
            let new_node_payload =
                put_substate_changes(store, current_version, &node_key, partition_changes);
            IdChange::new(node_key, new_node_payload)
        })
        .collect();
    put_node_changes(store, current_version, node_changes)
}

// only internals below

fn index_by_node_key_and_partition_num(
    changes: Vec<SubstateHashChange>,
) -> IndexMap<DbNodeKey, IndexMap<DbPartitionNum, Vec<IdChange<DbSortKey, Hash>>>> {
    let mut by_db_node_key = index_map_new();
    for IdChange { id, changed } in changes {
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
            .push(IdChange::new(sort_key, changed));
    }
    by_db_node_key
}

fn put_substate_changes<S: TreeStore<NodePayload> + TreeStore<SubstatePayload>>(
    store: &mut S,
    current_version: Option<Version>,
    node_key: &DbNodeKey,
    partition_changes: IndexMap<DbPartitionNum, Vec<IdChange<DbSortKey, Hash>>>,
) -> Option<NodePayload> {
    let mut partitions = get_node_payload(store, current_version, node_key)
        .map(|node_payload| node_payload.partitions)
        .unwrap_or_else(|| BTreeMap::new());
    for (partition_num, changes) in partition_changes {
        let db_partition_key = DbPartitionKey {
            node_key: node_key.clone(),
            partition_num,
        };
        let partition_state_version = partitions
            .remove(&partition_num)
            .map(|partition| partition.state_version);
        let mut partition_store = NestedTreeStore::new(store, db_partition_key);
        let new_partition_state_version = current_version.unwrap_or(0) + 1;
        let new_partition_root_hash = put_changes(
            &mut partition_store,
            partition_state_version,
            new_partition_state_version,
            changes
                .into_iter()
                .map(|change| to_substate_change(change))
                .collect(),
        );
        if new_partition_root_hash != SPARSE_MERKLE_PLACEHOLDER_HASH {
            partitions.insert(
                partition_num,
                PartitionPayload {
                    state_version: new_partition_state_version,
                    root_hash: new_partition_root_hash,
                },
            );
        }
    }
    if partitions.is_empty() {
        None
    } else {
        Some(NodePayload { partitions })
    }
}

fn put_node_changes<S: TreeStore<NodePayload>>(
    store: &mut S,
    current_version: Option<Version>,
    changes: Vec<IdChange<DbNodeKey, NodePayload>>,
) -> Hash {
    put_changes(
        store,
        current_version,
        current_version.unwrap_or(0) + 1,
        changes
            .into_iter()
            .map(|change| to_node_change(change))
            .collect(),
    )
}

fn get_node_payload<S: ReadableTreeStore<NodePayload>>(
    store: &S,
    current_version: Option<Version>,
    node_key: &DbNodeKey,
) -> Option<NodePayload> {
    let Some(current_version) = current_version else {
        return None;
    };
    JellyfishMerkleTree::new(store)
        .get_with_proof(&LeafKey::new(node_key), current_version)
        .unwrap()
        .0
        .map(|(_hash, payload, _version)| payload)
}

struct LeafChange<P> {
    key: LeafKey,
    new_payload: Option<(Hash, P)>,
}

fn put_changes<S: TreeStore<P>, P: Payload>(
    store: &mut S,
    current_version: Option<Version>,
    next_version: Version,
    changes: Vec<LeafChange<P>>,
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

fn to_node_change(change: IdChange<DbNodeKey, NodePayload>) -> LeafChange<NodePayload> {
    LeafChange {
        key: LeafKey::new(&change.id),
        new_payload: change
            .changed
            .map(|payload| (calculate_node_hash(&payload), payload)),
    }
}

fn calculate_node_hash(node_payload: &NodePayload) -> Hash {
    let mut buffer = Vec::with_capacity(node_payload.partitions.len() * (1 + Hash::LENGTH));
    for (partition_num, partition_payload) in &node_payload.partitions {
        buffer.push(*partition_num);
        buffer.extend_from_slice(&partition_payload.root_hash.0)
    }
    hash(&buffer)
}

fn to_substate_change(change: IdChange<DbSortKey, Hash>) -> LeafChange<SubstatePayload> {
    LeafChange {
        key: LeafKey { bytes: change.id.0 },
        new_payload: change.changed.map(|value_hash| (value_hash, ())),
    }
}

struct NestedTreeStore<'s, S> {
    underlying: &'s mut S,
    parent_key: LeafKey,
}

impl<'s, S> NestedTreeStore<'s, S> {
    pub fn new(underlying: &'s mut S, db_partition_key: DbPartitionKey) -> NestedTreeStore<'s, S> {
        NestedTreeStore {
            underlying,
            parent_key: LeafKey {
                bytes: db_partition_key.into_bytes(),
            },
        }
    }

    fn prefixed(&self, key: &NodeKey) -> NodeKey {
        NodeKey::new(
            key.version(),
            NibblePath::from_iter(
                NibblePath::new_even(self.parent_key.bytes.clone())
                    .nibbles()
                    .chain(key.nibble_path().nibbles()),
            ),
        )
    }
}

impl<'s, S: ReadableTreeStore<SubstatePayload>> ReadableTreeStore<SubstatePayload>
    for NestedTreeStore<'s, S>
{
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<SubstatePayload>> {
        self.underlying.get_node(&self.prefixed(key))
    }
}

impl<'s, S: WriteableTreeStore<SubstatePayload>> WriteableTreeStore<SubstatePayload>
    for NestedTreeStore<'s, S>
{
    fn insert_node(&mut self, key: NodeKey, node: TreeNode<SubstatePayload>) {
        self.underlying.insert_node(self.prefixed(&key), node);
    }

    fn record_stale_node(&mut self, key: NodeKey) {
        self.underlying.record_stale_node(self.prefixed(&key));
    }
}
