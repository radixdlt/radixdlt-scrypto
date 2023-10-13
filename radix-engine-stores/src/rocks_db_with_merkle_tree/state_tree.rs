use crate::hash_tree::put_at_next_version;
use crate::hash_tree::tree_store::{
    NodeKey, ReadableTreeStore, StaleTreePart, TreeNode, WriteableTreeStore,
};
use radix_engine_common::prelude::Hash;
use radix_engine_store_interface::interface::DatabaseUpdates;

struct CollectingTreeStore<'s, S> {
    readable_delegate: &'s S,
    diff: StateHashTreeDiff,
}

impl<'s, S: ReadableTreeStore> CollectingTreeStore<'s, S> {
    pub fn new(readable_delegate: &'s S) -> Self {
        Self {
            readable_delegate,
            diff: StateHashTreeDiff::new(),
        }
    }

    pub fn into_diff(self) -> StateHashTreeDiff {
        self.diff
    }
}

impl<'s, S: ReadableTreeStore> ReadableTreeStore for CollectingTreeStore<'s, S> {
    fn get_node(&self, key: &NodeKey) -> Option<TreeNode> {
        self.readable_delegate.get_node(key)
    }
}

impl<'s, S> WriteableTreeStore for CollectingTreeStore<'s, S> {
    fn insert_node(&mut self, key: NodeKey, node: TreeNode) {
        self.diff.new_nodes.push((key, node));
    }

    fn record_stale_tree_part(&mut self, part: StaleTreePart) {
        self.diff.stale_tree_parts.push(part);
    }
}

#[derive(Clone)]
pub struct StateHashTreeDiff {
    pub new_nodes: Vec<(NodeKey, TreeNode)>,
    pub stale_tree_parts: Vec<StaleTreePart>,
}

impl StateHashTreeDiff {
    pub fn new() -> Self {
        Self {
            new_nodes: Vec::new(),
            stale_tree_parts: Vec::new(),
        }
    }
}

pub fn compute_state_tree_update<S: ReadableTreeStore>(
    store: &S,
    parent_state_version: u64,
    database_updates: &DatabaseUpdates,
) -> (StateHashTreeDiff, Hash) {
    let mut collector = CollectingTreeStore::new(store);
    let root_hash = put_at_next_version(
        &mut collector,
        Some(parent_state_version).filter(|v| *v > 0),
        database_updates,
    );
    (collector.into_diff(), root_hash)
}
