use crate::hash_tree::tree_store::IndexPayload;
use crate::hash_tree::types::LeafKey;
use jellyfish::JellyfishMerkleTree;
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::*;
use sbor::rust::collections::{index_map_new, IndexMap};
use sbor::rust::vec::Vec;
use tree_store::{Payload, ReadableTreeStore, TreeNode, TreeStore, WriteableTreeStore};
use types::{NibblePath, NodeKey, Version};

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

/// Part of the ID addressing the upper layer of the JMT.
pub type DbIndex = Vec<u8>;

/// Part of the ID addressing the lower layer of the JMT.
pub type DbKey = Vec<u8>;

/// A complete ID of the substate, as tracked by the JMT.
pub struct DbId {
    index: DbIndex,
    key: DbKey,
}

impl DbId {
    pub fn new(index: DbIndex, key: DbKey) -> Self {
        Self { index, key }
    }
}

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
pub type SubstateHashChange = IdChange<DbId, Hash>;

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
pub fn put_at_next_version<S: TreeStore<IndexPayload> + TreeStore<()>>(
    store: &mut S,
    current_version: Option<Version>,
    changes: Vec<SubstateHashChange>,
) -> Hash {
    let changes_by_index = aggregate_by_db_index(changes);
    let mut nested_root_changes = Vec::new();
    for (index, substate_changes) in changes_by_index {
        let nested_root = put_substate_changes(store, current_version, &index, substate_changes);
        nested_root_changes.push(IdChange::new(index, nested_root));
    }
    put_index_changes(store, current_version, nested_root_changes)
}

// only internals below

fn aggregate_by_db_index(
    changes: Vec<SubstateHashChange>,
) -> IndexMap<DbIndex, Vec<IdChange<DbKey, Hash>>> {
    let mut by_db_index = index_map_new();
    for change in changes {
        let db_id = change.id;
        by_db_index
            .entry(db_id.index)
            .or_insert_with(|| Vec::new())
            .push(IdChange::new(db_id.key, change.changed));
    }
    by_db_index
}

#[derive(Debug)]
struct TreeRoot<P> {
    hash: Hash,
    node: TreeNode<P>,
}

fn put_substate_changes<S: TreeStore<IndexPayload> + TreeStore<()>>(
    store: &mut S,
    current_version: Option<Version>,
    index: &DbIndex,
    changes: Vec<IdChange<DbKey, Hash>>,
) -> Option<TreeRoot<()>> {
    let (subtree_last_update_state_version, subtree_root) =
        get_index_leaf_entry(store, current_version, index);
    let mut subtree_store = NestedTreeStore::new(store, index, subtree_root);
    let substate_root_hash = put_changes(
        &mut subtree_store,
        subtree_last_update_state_version,
        current_version.unwrap_or(0) + 1,
        changes
            .into_iter()
            .map(|change| to_substate_change(change))
            .collect(),
    );
    let substate_root_node = subtree_store.extract_new_root();
    if matches!(substate_root_node, TreeNode::Null) {
        None
    } else {
        Some(TreeRoot {
            hash: substate_root_hash,
            node: substate_root_node,
        })
    }
}

fn put_index_changes<S: TreeStore<IndexPayload>>(
    store: &mut S,
    current_version: Option<Version>,
    changes: Vec<IdChange<DbIndex, TreeRoot<()>>>,
) -> Hash {
    put_changes(
        store,
        current_version,
        current_version.unwrap_or(0) + 1,
        changes
            .into_iter()
            .map(|change| to_index_change(change))
            .collect(),
    )
}

fn get_index_leaf_entry<S: ReadableTreeStore<IndexPayload>>(
    store: &S,
    current_version: Option<Version>,
    index: &DbIndex,
) -> (Option<Version>, Option<TreeNode<()>>) {
    let Some(current_version) = current_version else {
        return (None, None);
    };
    let (node_option, _proof) = JellyfishMerkleTree::new(store)
        .get_with_proof(&LeafKey::new(index.as_slice()), current_version)
        .unwrap();

    let Some((_hash, payload, version)) = node_option else {
        return (None, None);
    };

    (Some(version), Some(payload))
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

fn to_index_change(change: IdChange<DbIndex, TreeRoot<()>>) -> LeafChange<IndexPayload> {
    LeafChange {
        key: LeafKey { bytes: change.id },
        new_payload: change.changed.map(|root| (root.hash, root.node)),
    }
}

fn to_substate_change(change: IdChange<DbKey, Hash>) -> LeafChange<()> {
    LeafChange {
        key: LeafKey { bytes: change.id },
        new_payload: change.changed.map(|value_hash| (value_hash, ())),
    }
}

struct NestedTreeStore<'s, S> {
    underlying: &'s mut S,
    parent_key: LeafKey,
    current_root: Option<TreeNode<()>>,
    new_root: Option<TreeNode<()>>,
}

impl<'s, S> NestedTreeStore<'s, S> {
    pub fn new(
        underlying: &'s mut S,
        index: &DbIndex,
        root: Option<TreeNode<()>>,
    ) -> NestedTreeStore<'s, S> {
        NestedTreeStore {
            underlying,
            parent_key: LeafKey::new(index),
            current_root: root,
            new_root: None,
        }
    }

    pub fn extract_new_root(&mut self) -> TreeNode<()> {
        self.new_root
            .take()
            .expect("no new root stored into the nested tree")
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

impl<'s, S: ReadableTreeStore<()>> ReadableTreeStore<()> for NestedTreeStore<'s, S> {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode<()>> {
        if key.nibble_path().is_empty() {
            self.current_root.clone()
        } else {
            self.underlying.get_node(&self.prefixed(key))
        }
    }
}

impl<'s, S: WriteableTreeStore<()>> WriteableTreeStore<()> for NestedTreeStore<'s, S> {
    fn insert_node(&mut self, key: NodeKey, node: TreeNode<()>) {
        if key.nibble_path().is_empty() {
            self.new_root = Some(node);
        } else {
            self.underlying.insert_node(self.prefixed(&key), node);
        }
    }

    fn record_stale_node(&mut self, key: NodeKey) {
        if key.nibble_path().is_empty() {
            self.current_root = None;
        } else {
            self.underlying.record_stale_node(self.prefixed(&key));
        }
    }
}
