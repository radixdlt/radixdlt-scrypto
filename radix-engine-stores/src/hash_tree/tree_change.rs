use crate::hash_tree::{BatchChange, HashChange, IdHashChange};
use radix_engine_common::crypto::Hash;
use radix_engine_store_interface::interface::{
    DbNodeKey, DbPartitionKey, DbPartitionNum, DbSortKey,
};
use utils::prelude::{index_map_new, IndexMap};
use utils::rust::vec::Vec;

/// A canonical form of "change to an entire hash tree".
/// Use `From<Vec<HashChange>>` implementation to canonicalize a sequence of changes.
pub enum TreeChange {
    /// A "delta" - the only supported type of a global change.
    /// Other global changes are conceivable (e.g. "clear entire tree"), but not needed.
    Delta {
        node_changes: Vec<(DbNodeKey, NodeTierChange)>,
    },
}

/// A change to a specific ReNode's part of the hash tree.
pub enum NodeTierChange {
    /// A "delta" - the only supported type of ReNode-tier change.
    /// Other Node-tier changes are conceivable (e.g. "delete entire ReNode"), but not needed.
    Delta {
        partition_changes: Vec<(DbPartitionNum, PartitionTierChange)>,
    },
}

/// A change to a specific Partition's part of the hash tree.
pub enum PartitionTierChange {
    /// A reset, i.e. complete replacement of partition's contents.
    /// It may denote a partition delete (when `substate_hashes.is_empty()`).
    Reset {
        substate_hashes: Vec<(DbSortKey, Hash)>,
    },
    /// A "delta", i.e. some set of changes (set or delete - see [`IdHashChange`]) to individual
    /// substates.
    Delta {
        substate_changes: Vec<IdHashChange<DbSortKey>>,
    },
}

// only internals below

impl From<Vec<HashChange>> for TreeChange {
    fn from(changes: Vec<HashChange>) -> Self {
        let mut builder = TreeChangeBuilder::default();
        for change in changes {
            match change {
                HashChange::Single(single) => {
                    let IdHashChange {
                        id:
                            (
                                DbPartitionKey {
                                    node_key,
                                    partition_num,
                                },
                                sort_key,
                            ),
                        hash_change,
                    } = single;
                    builder
                        .node(node_key)
                        .partition(partition_num)
                        .apply_substate_change(sort_key, hash_change);
                }
                HashChange::Batch(batch) => match batch {
                    BatchChange::DeletePartition(DbPartitionKey {
                        node_key,
                        partition_num,
                    }) => {
                        builder
                            .node(node_key)
                            .partition(partition_num)
                            .apply_delete();
                    }
                },
            }
        }
        builder.build()
    }
}

#[derive(Default)]
struct TreeChangeBuilder {
    node_builders: IndexMap<DbNodeKey, NodeTierChangeBuilder>,
}

impl TreeChangeBuilder {
    pub fn node(&mut self, node_key: DbNodeKey) -> &mut NodeTierChangeBuilder {
        self.node_builders
            .entry(node_key)
            .or_insert_with(|| NodeTierChangeBuilder::default())
    }

    pub fn build(self) -> TreeChange {
        TreeChange::Delta {
            node_changes: self
                .node_builders
                .into_iter()
                .map(|(key, builder)| (key, builder.build()))
                .collect(),
        }
    }
}

#[derive(Default)]
struct NodeTierChangeBuilder {
    partition_builders: IndexMap<DbPartitionNum, PartitionTierChangeBuilder>,
}

impl NodeTierChangeBuilder {
    pub fn partition(&mut self, partition_num: DbPartitionNum) -> &mut PartitionTierChangeBuilder {
        self.partition_builders
            .entry(partition_num)
            .or_insert_with(|| PartitionTierChangeBuilder::default())
    }

    pub fn build(self) -> NodeTierChange {
        NodeTierChange::Delta {
            partition_changes: self
                .partition_builders
                .into_iter()
                .map(|(key, builder)| (key, builder.build()))
                .collect(),
        }
    }
}

#[derive(Default)]
struct PartitionTierChangeBuilder {
    delete_previous_contents: bool,
    changed_hashes: IndexMap<DbSortKey, Option<Hash>>,
}

impl PartitionTierChangeBuilder {
    pub fn apply_substate_change(&mut self, sort_key: DbSortKey, hash_change: Option<Hash>) {
        self.changed_hashes.insert(sort_key, hash_change);
    }

    pub fn apply_delete(&mut self) {
        self.delete_previous_contents = true;
        self.changed_hashes = index_map_new();
    }

    pub fn build(self) -> PartitionTierChange {
        if self.delete_previous_contents {
            PartitionTierChange::Reset {
                substate_hashes: self
                    .changed_hashes
                    .into_iter()
                    .map(|(key, change)| {
                        let Some(hash) = change else {
                            panic!("inconsistent change: {:?} deleted after partition delete", key);
                        };
                        (key, hash)
                    })
                    .collect(),
            }
        } else {
            PartitionTierChange::Delta {
                substate_changes: self
                    .changed_hashes
                    .into_iter()
                    .map(|(key, change)| IdHashChange::new(key, change))
                    .collect(),
            }
        }
    }
}
