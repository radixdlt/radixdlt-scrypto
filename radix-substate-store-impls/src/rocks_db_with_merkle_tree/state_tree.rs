use crate::state_tree::put_at_next_version;
use crate::state_tree::tree_store::*;
use radix_common::prelude::Hash;
use radix_substate_store_interface::interface::{DatabaseUpdates, DbPartitionKey, DbSortKey};
use std::cell::RefCell;

struct CollectingTreeStore<'s, S> {
    readable_delegate: &'s S,
    diff: StateTreeDiff,
}

impl<'s, S: ReadableTreeStore> CollectingTreeStore<'s, S> {
    pub fn new(readable_delegate: &'s S) -> Self {
        Self {
            readable_delegate,
            diff: StateTreeDiff::new(),
        }
    }

    pub fn into_diff(self) -> StateTreeDiff {
        self.diff
    }
}

impl<'s, S: ReadableTreeStore> ReadableTreeStore for CollectingTreeStore<'s, S> {
    fn get_node(&self, key: &StoredTreeNodeKey) -> Option<TreeNode> {
        self.readable_delegate.get_node(key)
    }
}

impl<'s, S> WriteableTreeStore for CollectingTreeStore<'s, S> {
    fn insert_node(&self, key: StoredTreeNodeKey, node: TreeNode) {
        self.diff.new_nodes.borrow_mut().push((key, node));
    }

    fn associate_substate(
        &self,
        _state_tree_leaf_key: &StoredTreeNodeKey,
        _partition_key: &DbPartitionKey,
        _sort_key: &DbSortKey,
        _substate_value: AssociatedSubstateValue,
    ) {
        // intentionally empty
    }

    fn record_stale_tree_part(&self, part: StaleTreePart) {
        self.diff.stale_tree_parts.borrow_mut().push(part);
    }
}

#[derive(Clone)]
pub struct StateTreeDiff {
    pub new_nodes: RefCell<Vec<(StoredTreeNodeKey, TreeNode)>>,
    pub stale_tree_parts: RefCell<Vec<StaleTreePart>>,
}

impl StateTreeDiff {
    pub fn new() -> Self {
        Self {
            new_nodes: RefCell::new(Vec::new()),
            stale_tree_parts: RefCell::new(Vec::new()),
        }
    }
}

pub fn compute_state_tree_update<S: ReadableTreeStore>(
    store: &S,
    parent_state_version: u64,
    database_updates: &DatabaseUpdates,
) -> (StateTreeDiff, Hash) {
    let mut collector = CollectingTreeStore::new(store);
    let root_hash = put_at_next_version(
        &mut collector,
        Some(parent_state_version).filter(|v| *v > 0),
        database_updates,
    );
    (collector.into_diff(), root_hash)
}
